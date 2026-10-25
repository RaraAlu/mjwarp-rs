// QA 独立复核 · G5 设备探针：**QA 自己的 launcher**（不复用交付 probe.cu / gpu_compare.rs）。
//
// 做法：`#include` 产品 device 源（产品只暴露 NVRTC 源码，没有 launcher），由本文件自写
//   kernel 包装 + host 端装载/启动/回读；输入是本目录 `qa-gpu/cases/<case>/`（QA 自己在
//   GPU 上采的位姿 + 模型数组）或交付窗口气泡以外的自造场景；输出写回 `<out>/<case>/`。
//
// 模式（meta.txt 的 kind）：contact（CONVEX ccd 写入）/ sdf / hfield / witness（ccd() 直调）。
// 每个 world 一次 launch（grid.x = nworld），每 world 独立 polytope 缓冲，避免共享竞态。
//
// 编译：nvcc -std=c++17 -O3 -arch=sm_89 --fmad=true -I src/physics/collision/cuda
//       -L <cudart lib/x64> -o target/qa-g5/qa_probe.exe <本文件>
// 用法：qa_probe.exe <cases_root> <out_root> <cases.txt> [run_tag]

#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <fstream>
#include <iterator>
#include <map>
#include <sstream>
#include <string>
#include <vector>

#include <cuda_runtime.h>

#include "contact.cu"
#include "sdf_narrowphase.cu"

// ---------------- host 侧 case 描述（POD 传给 device） ----------------

struct QaCase {
  int nworld, ngeom, naconmax, naccdmax, efc_width, g1, g2, type1, type2, kind;
  int gjk_iterations, epa_iterations, sdf_initpoints, sdf_iterations;
  float tolerance, margin, gap;
  int condim;
  float friction[5], solref[2], solreffriction[2], solimp[5], adhesion;
  int pairid0, pairid1;
  int oct_root, hfield_id, noct, nmeshvert, nmeshgraph;
  int max_horizon;
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
};

struct QaOut {
  float* dist;
  float* pos;
  float* frame;
  float* includemargin;
  float* friction;
  float* solref;
  float* solreffriction;
  float* solimp;
  int* dim;
  int* geom;
  int* efc_address;
  int* worldid;
  int* type;
  int* geomcollisionid;
  float* adhesion;
  int* nacon;
  int* nccd;
  int* prism_count;
  unsigned* overflow;
};

struct QaWitnessOut {
  float* dist;
  float* v1;
  float* v2;
  int* ncontact;
  int* index;
  unsigned* overflow;
};

// ---------------- device 侧构造 ----------------

__device__ __forceinline__ d_geom qa_make_geom(const QaCase* c, int geom_index, int world) {
  d_geom geom;
  const int ngeom = c->ngeom;
  const int base_pos = (world * ngeom + geom_index) * 3;
  const int base_rot = (world * ngeom + geom_index) * 9;
  geom.pos = v3_make(c->geom_xpos[base_pos], c->geom_xpos[base_pos + 1],
                     c->geom_xpos[base_pos + 2]);
  for (int k = 0; k < 9; ++k) geom.rot[k] = c->geom_xmat[base_rot + k];
  geom.size = v3_make(c->geom_size[geom_index * 3], c->geom_size[geom_index * 3 + 1],
                      c->geom_size[geom_index * 3 + 2]);
  geom.margin = c->geom_margin[geom_index];
  for (int k = 0; k < 18; ++k) geom.polyvert[k] = 0.0f;
  geom.vertadr = -1;
  geom.vertnum = -1;
  geom.graphadr = -1;
  geom.index = -1;
  geom.mesh.vert = nullptr;
  geom.mesh.graph = nullptr;
  const int type = c->geom_type[geom_index];
  if (type == G5_MESH) {
    const int dataid = c->geom_dataid[geom_index];
    geom.vertadr = c->mesh_vertadr[dataid];
    geom.vertnum = c->mesh_vertnum[dataid];
    geom.graphadr = c->mesh_graphadr[dataid];
    geom.mesh.vert = c->mesh_vert;
    geom.mesh.graph = c->mesh_graph;
  }
  return geom;
}

__device__ __forceinline__ int qa_sdf_type_of(int mjt_type) {
  switch (mjt_type) {
    case G5_PLANE: return G5_SDF_PLANE;
    case G5_SPHERE: return G5_SDF_SPHERE;
    case G5_CAPSULE: return G5_SDF_CAPSULE;
    case G5_ELLIPSOID: return G5_SDF_ELLIPSOID;
    case G5_CYLINDER: return G5_SDF_CYLINDER;
    case G5_BOX: return G5_SDF_BOX;
    case G5_MESH: return G5_SDF_MESH;
    case G5_SDF: return G5_SDF_SDF;
    default: return G5_SDF_SPHERE;
  }
}

__device__ __forceinline__ d_contact_params qa_make_params(const QaCase* c, int world) {
  d_contact_params params;
  params.margin = c->margin;
  params.gap = c->gap;
  params.condim = c->condim;
  for (int k = 0; k < 5; ++k) params.friction[k] = c->friction[k];
  for (int k = 0; k < 5; ++k) params.solimp[k] = c->solimp[k];
  for (int k = 0; k < 2; ++k) params.solref[k] = c->solref[k];
  for (int k = 0; k < 2; ++k) params.solreffriction[k] = c->solreffriction[k];
  params.adhesion = c->adhesion;
  params.geoms[0] = c->g1;
  params.geoms[1] = c->g2;
  params.pairid[0] = c->pairid0;
  params.pairid[1] = c->pairid1;
  params.worldid = world;
  return params;
}

__device__ __forceinline__ d_contact_out qa_make_out(const QaCase* c, QaOut* o) {
  d_contact_out out;
  out.dist = o->dist;
  out.pos = (v3*)o->pos;
  out.frame = o->frame;
  out.includemargin = o->includemargin;
  out.friction = o->friction;
  out.solref = o->solref;
  out.solreffriction = o->solreffriction;
  out.solimp = o->solimp;
  out.dim = o->dim;
  out.geom = o->geom;
  out.efc_address = o->efc_address;
  out.worldid = o->worldid;
  out.type = o->type;
  out.geomcollisionid = o->geomcollisionid;
  out.adhesion = o->adhesion;
  out.nacon = o->nacon;
  out.efc_address_width = c->efc_width;
  return out;
}

// polytope 容量按 epa_iterations 上界（64）取常量；算法只用 epa_iterations 次迭代。
#define QA_MAX_EPA_ITER 64
#define QA_MAX_VERT (10 + 2 * QA_MAX_EPA_ITER)
#define QA_MAX_FACE (6 + 5 * QA_MAX_EPA_ITER)
#define QA_MAX_HORIZON 24
#define QA_MAX_SDF_CONTACTS 64

__device__ __forceinline__ void qa_init_polytope(d_polytope* pt, v3* vert, int* vert_index,
                                                 unsigned* face, v3* face_pr, float* face_norm2,
                                                 int* horizon, int max_horizon) {
  pt->nvert = 0;
  pt->nface = 0;
  pt->nhorizon = 0;
  pt->status = 0;
  pt->vert = vert;
  pt->vert_index = vert_index;
  pt->face = face;
  pt->face_pr = face_pr;
  pt->face_norm2 = face_norm2;
  pt->horizon = horizon;
  pt->max_face = QA_MAX_FACE;
  pt->max_horizon = max_horizon;
  pt->center = v3_make(0.0f, 0.0f, 0.0f);
}

__global__ void qa_contact_kernel(const QaCase* c, QaOut* o) {
  const int world = blockIdx.x;
  const int kind = c->kind;  // 0=convex 1=sdf 2=hfield
  d_geom g1 = qa_make_geom(c, c->g1, world);
  d_geom g2 = qa_make_geom(c, c->g2, world);
  d_contact_params params = qa_make_params(c, world);
  d_contact_out out = qa_make_out(c, o);

  v3 vert[QA_MAX_VERT];
  int vert_index[QA_MAX_VERT];
  unsigned face[QA_MAX_FACE];
  v3 face_pr[QA_MAX_FACE];
  float face_norm2[QA_MAX_FACE];
  int horizon[QA_MAX_HORIZON];
  d_polytope pt;
  qa_init_polytope(&pt, vert, vert_index, face, face_pr, face_norm2, horizon, c->max_horizon);

  if (kind == 0) {
    d_contact_pair_input in;
    in.geom1 = g1;
    in.geom2 = g2;
    in.geomtype1 = c->type1;
    in.geomtype2 = c->type2;
    in.tolerance = c->tolerance;
    in.gjk_iterations = c->gjk_iterations;
    in.epa_iterations = c->epa_iterations;
    in.collision_sensor = false;
    in.naccdmax = c->naccdmax;
    in.geomgeomid = 0;
    in.overflow_out = o->overflow;
    g5_ccd_pair_write(in, params, c->naconmax, o->nccd, &pt, &out);
    return;
  }

  if (kind == 1) {
    d_volume_data volume;
    const int root = c->oct_root;
    volume.center = v3_make(c->oct_aabb[root * 6 + 0], c->oct_aabb[root * 6 + 1],
                            c->oct_aabb[root * 6 + 2]);
    volume.half_size = v3_make(c->oct_aabb[root * 6 + 3], c->oct_aabb[root * 6 + 4],
                               c->oct_aabb[root * 6 + 5]);
    volume.oct_aabb = c->oct_aabb;
    volume.oct_child = c->oct_child;
    volume.oct_coeff = c->oct_coeff;
    volume.root = root;
    volume.noct = c->noct;
    d_sdf_shape shape1;
    shape1.geom = qa_sdf_type_of(c->type1);
    shape1.size = g1.size;
    shape1.volume = nullptr;
    shape1.has_volume = false;
    d_sdf_shape shape2;
    shape2.geom = qa_sdf_type_of(c->type2);
    shape2.size = g2.size;
    shape2.volume = &volume;
    shape2.has_volume = true;
    d_sdf_pair pair;
    pair.shape1 = shape1;
    pair.shape2 = shape2;
    pair.pos1 = g1.pos;
    for (int k = 0; k < 9; ++k) pair.rot1[k] = g1.rot[k];
    pair.pos2 = g2.pos;
    for (int k = 0; k < 9; ++k) pair.rot2[k] = g2.rot[k];
    for (int k = 0; k < 6; ++k) pair.aabb1[k] = c->geom_aabb[c->g1 * 6 + k];
    for (int k = 0; k < 6; ++k) pair.aabb2[k] = c->geom_aabb[c->g2 * 6 + k];
    pair.sdf_initpoints = c->sdf_initpoints;
    pair.sdf_iterations = c->sdf_iterations;
    d_sdf_contact contacts[QA_MAX_SDF_CONTACTS];
    const int count = g5_sdf_pair_contacts(&pair, contacts, QA_MAX_SDF_CONTACTS);
    for (int i = 0; i < count; ++i) {
      g5_write_contact(params, c->naconmax, 0, contacts[i].dist, contacts[i].pos,
                       contacts[i].frame, &out);
    }
    return;
  }

  // hfield：geom1 恒为 hfield（位姿为世界系），geom2 为对侧 geom
  d_hfield_data hf;
  for (int k = 0; k < 4; ++k) hf.size[k] = c->hfield_size[k];
  hf.nrow = c->hfield_nrow[0];
  hf.ncol = c->hfield_ncol[0];
  hf.adr = c->hfield_adr[0];
  hf.data = c->hfield_data;
  d_ccd_options options;
  options.tolerance = c->tolerance;
  options.cutoff = 0.0f;
  options.gjk_iterations = c->gjk_iterations;
  options.epa_iterations = c->epa_iterations;
  d_hfield_result result;
  g5_hfield_pair_contacts(hf, g1.pos, g1.rot, g2, c->type2, c->geom_rbound[c->g2], c->margin,
                          c->gap, options, &pt, &result);
  o->prism_count[world] = result.prism_count;
  g5_hfield_write_contacts(result, params, c->naconmax, o->overflow, &out);
}

__global__ void qa_witness_kernel(const QaCase* c, QaWitnessOut* o) {
  const int world = blockIdx.x;
  d_geom g1 = qa_make_geom(c, c->g1, world);
  d_geom g2 = qa_make_geom(c, c->g2, world);
  g1.margin = c->margin;
  g2.margin = c->margin;
  const v3 origin = g1.pos;
  g1.pos = v3_make(0.0f, 0.0f, 0.0f);
  g2.pos = v3_sub(g2.pos, origin);
  v3 vert[QA_MAX_VERT];
  int vert_index[QA_MAX_VERT];
  unsigned face[QA_MAX_FACE];
  v3 face_pr[QA_MAX_FACE];
  float face_norm2[QA_MAX_FACE];
  int horizon[QA_MAX_HORIZON];
  d_polytope pt;
  qa_init_polytope(&pt, vert, vert_index, face, face_pr, face_norm2, horizon, c->max_horizon);
  d_ccd_options options;
  options.tolerance = c->tolerance;
  options.cutoff = c->gap;
  options.gjk_iterations = c->gjk_iterations;
  options.epa_iterations = c->epa_iterations;
  d_ccd_result result;
  g5_ccd(g1, g2, c->type1, c->type2, g1.pos, g2.pos, options, &pt, &result);
  o->dist[world] = result.dist;
  o->v1[world * 3 + 0] = result.x1.x;
  o->v1[world * 3 + 1] = result.x1.y;
  o->v1[world * 3 + 2] = result.x1.z;
  o->v2[world * 3 + 0] = result.x2.x;
  o->v2[world * 3 + 1] = result.x2.y;
  o->v2[world * 3 + 2] = result.x2.z;
  o->ncontact[world] = result.ncontact;
  o->index[world] = result.multiccd_idx;
  o->overflow[world] = result.overflow;
}

// ---------------- host 工具 ----------------

static void die(const char* message) {
  std::fprintf(stderr, "qa_probe: %s\n", message);
  std::exit(2);
}

static void check_cuda(const char* where) {
  cudaError_t error = cudaGetLastError();
  if (error != cudaSuccess) {
    std::fprintf(stderr, "qa_probe: CUDA error at %s: %s\n", where, cudaGetErrorString(error));
    std::exit(3);
  }
}

static std::vector<char> read_file(const std::string& path) {
  std::ifstream stream(path, std::ios::binary);
  if (!stream) die(("cannot open " + path).c_str());
  return std::vector<char>((std::istreambuf_iterator<char>(stream)),
                           std::istreambuf_iterator<char>());
}

template <typename T>
static std::vector<T> read_bin(const std::string& path) {
  std::vector<char> bytes = read_file(path);
  if (bytes.size() % sizeof(T) != 0) die(("bad bin size " + path).c_str());
  std::vector<T> values(bytes.size() / sizeof(T));
  std::memcpy(values.data(), bytes.data(), bytes.size());
  return values;
}

static void write_bin(const std::string& path, const void* data, size_t bytes) {
  std::ofstream stream(path, std::ios::binary);
  if (!stream) die(("cannot write " + path).c_str());
  stream.write((const char*)data, (std::streamsize)bytes);
}

static std::map<std::string, std::vector<std::string>> read_meta(const std::string& path) {
  std::ifstream stream(path);
  if (!stream) die(("cannot open " + path).c_str());
  std::map<std::string, std::vector<std::string>> meta;
  std::string line;
  while (std::getline(stream, line)) {
    std::istringstream parts(line);
    std::string key;
    parts >> key;
    if (key.empty()) continue;
    std::vector<std::string> values;
    std::string value;
    while (parts >> value) values.push_back(value);
    meta[key] = values;
  }
  return meta;
}

static float meta_f32(const std::map<std::string, std::vector<std::string>>& meta,
                      const std::string& key) {
  return std::strtof(meta.at(key)[0].c_str(), nullptr);
}
static int meta_i32(const std::map<std::string, std::vector<std::string>>& meta,
                    const std::string& key) {
  return std::atoi(meta.at(key)[0].c_str());
}
static std::string meta_str(const std::map<std::string, std::vector<std::string>>& meta,
                            const std::string& key) {
  return meta.at(key)[0];
}

template <typename T>
static T* upload(const std::vector<T>& values) {
  void* device = nullptr;
  if (!values.empty()) {
    cudaMalloc(&device, values.size() * sizeof(T));
    cudaMemcpy(device, values.data(), values.size() * sizeof(T), cudaMemcpyHostToDevice);
    check_cuda("upload");
  }
  return (T*)device;
}

template <typename T>
static void download(T* device, std::vector<T>& host, const std::string& path) {
  if (!host.empty()) cudaMemcpy(host.data(), device, host.size() * sizeof(T),
                                cudaMemcpyDeviceToHost);
  check_cuda("download");
  if (!host.empty()) write_bin(path, host.data(), host.size() * sizeof(T));
}

static int kind_id(const std::string& kind) {
  if (kind == "contact") return 0;
  if (kind == "sdf") return 1;
  if (kind == "hfield") return 2;
  if (kind == "witness") return 3;
  die(("bad kind " + kind).c_str());
  return -1;
}

static void run_contact_case(const std::string& case_dir, const std::string& out_dir,
                             bool second_run) {
  auto meta = read_meta(case_dir + "/meta.txt");
  const std::string kind = meta_str(meta, "kind");
  const int id = kind_id(kind);
  QaCase c{};
  c.nworld = meta_i32(meta, "nworld");
  c.ngeom = meta_i32(meta, "ngeom");
  c.naconmax = meta_i32(meta, "naconmax");
  c.naccdmax = meta_i32(meta, "naccdmax");
  c.efc_width = meta_i32(meta, "efc_width");
  c.g1 = meta_i32(meta, "g1");
  c.g2 = meta_i32(meta, "g2");
  c.type1 = meta_i32(meta, "type1");
  c.type2 = meta_i32(meta, "type2");
  c.kind = id;
  c.gjk_iterations = meta_i32(meta, "gjk_iterations");
  c.epa_iterations = meta_i32(meta, "epa_iterations");
  c.sdf_initpoints = meta_i32(meta, "sdf_initpoints");
  c.sdf_iterations = meta_i32(meta, "sdf_iterations");
  c.tolerance = meta_f32(meta, "tolerance");
  c.margin = meta_f32(meta, "margin");
  c.gap = meta_f32(meta, "gap");
  c.condim = meta_i32(meta, "condim");
  for (int k = 0; k < 5; ++k) c.friction[k] = std::strtof(meta.at("friction")[k].c_str(), nullptr);
  for (int k = 0; k < 5; ++k) c.solimp[k] = std::strtof(meta.at("solimp")[k].c_str(), nullptr);
  for (int k = 0; k < 2; ++k) c.solref[k] = std::strtof(meta.at("solref")[k].c_str(), nullptr);
  for (int k = 0; k < 2; ++k)
    c.solreffriction[k] = std::strtof(meta.at("solreffriction")[k].c_str(), nullptr);
  c.adhesion = meta_f32(meta, "adhesion");
  c.pairid0 = meta_i32(meta, "pairid0");
  c.pairid1 = meta_i32(meta, "pairid1");
  c.oct_root = meta_i32(meta, "oct_root");
  c.hfield_id = meta_i32(meta, "hfield_id");
  c.noct = meta_i32(meta, "noct");
  c.nmeshvert = meta_i32(meta, "nmeshvert");
  c.nmeshgraph = meta_i32(meta, "nmeshgraph");
  c.max_horizon = meta.count("max_horizon") ? meta_i32(meta, "max_horizon") : 24;

  // 模型/位姿 bins
  auto geom_xpos = read_bin<float>(case_dir + "/geom_xpos.bin");
  auto geom_xmat = read_bin<float>(case_dir + "/geom_xmat.bin");
  auto geom_size = read_bin<float>(case_dir + "/geom_size.bin");
  auto geom_margin = read_bin<float>(case_dir + "/geom_margin.bin");
  auto geom_gap = read_bin<float>(case_dir + "/geom_gap.bin");
  auto geom_rbound = read_bin<float>(case_dir + "/geom_rbound.bin");
  auto geom_aabb = read_bin<float>(case_dir + "/geom_aabb.bin");
  auto geom_type = read_bin<int>(case_dir + "/geom_type.bin");
  auto geom_dataid = read_bin<int>(case_dir + "/geom_dataid.bin");
  auto mesh_vertadr = read_bin<int>(case_dir + "/mesh_vertadr.bin");
  auto mesh_vertnum = read_bin<int>(case_dir + "/mesh_vertnum.bin");
  auto mesh_graphadr = read_bin<int>(case_dir + "/mesh_graphadr.bin");
  auto mesh_octadr = read_bin<int>(case_dir + "/mesh_octadr.bin");
  auto mesh_vert = read_bin<float>(case_dir + "/mesh_vert.bin");
  auto mesh_graph = read_bin<int>(case_dir + "/mesh_graph.bin");
  auto hfield_size = read_bin<float>(case_dir + "/hfield_size.bin");
  auto hfield_nrow = read_bin<int>(case_dir + "/hfield_nrow.bin");
  auto hfield_ncol = read_bin<int>(case_dir + "/hfield_ncol.bin");
  auto hfield_adr = read_bin<int>(case_dir + "/hfield_adr.bin");
  auto hfield_data = read_bin<float>(case_dir + "/hfield_data.bin");
  auto oct_aabb = read_bin<float>(case_dir + "/oct_aabb.bin");
  auto oct_child = read_bin<int>(case_dir + "/oct_child.bin");
  auto oct_coeff = read_bin<float>(case_dir + "/oct_coeff.bin");

  QaCase* dc = (QaCase*)upload(std::vector<QaCase>{c});
  c.geom_xpos = upload(geom_xpos);
  c.geom_xmat = upload(geom_xmat);
  c.geom_size = upload(geom_size);
  c.geom_margin = upload(geom_margin);
  c.geom_gap = upload(geom_gap);
  c.geom_rbound = upload(geom_rbound);
  c.geom_aabb = upload(geom_aabb);
  c.geom_type = upload(geom_type);
  c.geom_dataid = upload(geom_dataid);
  c.mesh_vertadr = upload(mesh_vertadr);
  c.mesh_vertnum = upload(mesh_vertnum);
  c.mesh_graphadr = upload(mesh_graphadr);
  c.mesh_octadr = upload(mesh_octadr);
  c.mesh_vert = upload(mesh_vert);
  c.mesh_graph = upload(mesh_graph);
  c.hfield_size = upload(hfield_size);
  c.hfield_nrow = upload(hfield_nrow);
  c.hfield_ncol = upload(hfield_ncol);
  c.hfield_adr = upload(hfield_adr);
  c.hfield_data = upload(hfield_data);
  c.oct_aabb = upload(oct_aabb);
  c.oct_child = upload(oct_child);
  c.oct_coeff = upload(oct_coeff);
  cudaMemcpy(dc, &c, sizeof(QaCase), cudaMemcpyHostToDevice);
  check_cuda("case upload");

  const int worlds = c.nworld;
  QaOut out{};
  std::vector<float> dist(c.naconmax, 0.0f), pos((size_t)c.naconmax * 3, 0.0f),
      frame((size_t)c.naconmax * 9, 0.0f), includemargin(c.naconmax, 0.0f),
      friction((size_t)c.naconmax * 5, 0.0f), solref((size_t)c.naconmax * 2, 0.0f),
      solreffriction((size_t)c.naconmax * 2, 0.0f), solimp((size_t)c.naconmax * 5, 0.0f),
      adhesion(c.naconmax, 0.0f);
  std::vector<int> dim(c.naconmax, 0), geom((size_t)c.naconmax * 2, 0),
      efc((size_t)c.naconmax * c.efc_width, 0), worldid(c.naconmax, 0), type(c.naconmax, 0),
      geomcollisionid(c.naconmax, 0), nacon(1, 0), nccd(1, 0);
  std::vector<unsigned> overflow((size_t)worlds, 0u);
  std::vector<int> prism_count((size_t)worlds, 0);

  out.dist = upload(dist);
  out.pos = upload(pos);
  out.frame = upload(frame);
  out.includemargin = upload(includemargin);
  out.friction = upload(friction);
  out.solref = upload(solref);
  out.solreffriction = upload(solreffriction);
  out.solimp = upload(solimp);
  out.dim = upload(dim);
  out.geom = upload(geom);
  out.efc_address = upload(efc);
  out.worldid = upload(worldid);
  out.type = upload(type);
  out.geomcollisionid = upload(geomcollisionid);
  out.adhesion = upload(adhesion);
  out.nacon = upload(nacon);
  out.nccd = upload(nccd);
  out.prism_count = upload(prism_count);
  out.overflow = upload(overflow);
  QaOut* dout = (QaOut*)upload(std::vector<QaOut>{out});

  cudaMemset(out.nacon, 0, sizeof(int));
  cudaMemset(out.nccd, 0, sizeof(int));
  cudaMemset(out.overflow, 0, sizeof(unsigned) * worlds);
  cudaMemset(out.prism_count, 0, sizeof(int) * worlds);
  qa_contact_kernel<<<worlds, 1>>>(dc, dout);
  check_cuda("contact kernel");
  cudaDeviceSynchronize();
  check_cuda("contact kernel sync");

  std::vector<int> nacon_host(1, 0), nccd_host(1, 0);
  std::vector<unsigned> overflow_host((size_t)worlds, 0u);
  cudaMemcpy(nacon_host.data(), out.nacon, sizeof(int), cudaMemcpyDeviceToHost);
  cudaMemcpy(nccd_host.data(), out.nccd, sizeof(int), cudaMemcpyDeviceToHost);
  cudaMemcpy(overflow_host.data(), out.overflow, sizeof(unsigned) * worlds,
             cudaMemcpyDeviceToHost);
  check_cuda("read counters");

  cudaMemcpy(dist.data(), out.dist, sizeof(float) * c.naconmax, cudaMemcpyDeviceToHost);
  cudaMemcpy(pos.data(), out.pos, sizeof(float) * c.naconmax * 3, cudaMemcpyDeviceToHost);
  cudaMemcpy(frame.data(), out.frame, sizeof(float) * c.naconmax * 9, cudaMemcpyDeviceToHost);
  cudaMemcpy(includemargin.data(), out.includemargin, sizeof(float) * c.naconmax,
             cudaMemcpyDeviceToHost);
  cudaMemcpy(friction.data(), out.friction, sizeof(float) * c.naconmax * 5,
             cudaMemcpyDeviceToHost);
  cudaMemcpy(solref.data(), out.solref, sizeof(float) * c.naconmax * 2, cudaMemcpyDeviceToHost);
  cudaMemcpy(solreffriction.data(), out.solreffriction, sizeof(float) * c.naconmax * 2,
             cudaMemcpyDeviceToHost);
  cudaMemcpy(solimp.data(), out.solimp, sizeof(float) * c.naconmax * 5,
             cudaMemcpyDeviceToHost);
  cudaMemcpy(adhesion.data(), out.adhesion, sizeof(float) * c.naconmax, cudaMemcpyDeviceToHost);
  cudaMemcpy(dim.data(), out.dim, sizeof(int) * c.naconmax, cudaMemcpyDeviceToHost);
  cudaMemcpy(geom.data(), out.geom, sizeof(int) * c.naconmax * 2, cudaMemcpyDeviceToHost);
  cudaMemcpy(efc.data(), out.efc_address, sizeof(int) * c.naconmax * c.efc_width,
             cudaMemcpyDeviceToHost);
  cudaMemcpy(worldid.data(), out.worldid, sizeof(int) * c.naconmax, cudaMemcpyDeviceToHost);
  cudaMemcpy(type.data(), out.type, sizeof(int) * c.naconmax, cudaMemcpyDeviceToHost);
  cudaMemcpy(geomcollisionid.data(), out.geomcollisionid, sizeof(int) * c.naconmax,
             cudaMemcpyDeviceToHost);
  check_cuda("read contact arrays");

  const std::string tag = second_run ? "-run2" : "";
  write_bin(out_dir + "/dist" + tag + ".bin", dist.data(), sizeof(float) * dist.size());
  write_bin(out_dir + "/pos" + tag + ".bin", pos.data(), sizeof(float) * pos.size());
  write_bin(out_dir + "/frame" + tag + ".bin", frame.data(), sizeof(float) * frame.size());
  write_bin(out_dir + "/includemargin" + tag + ".bin", includemargin.data(),
            sizeof(float) * includemargin.size());
  write_bin(out_dir + "/friction" + tag + ".bin", friction.data(), sizeof(float) * friction.size());
  write_bin(out_dir + "/solref" + tag + ".bin", solref.data(), sizeof(float) * solref.size());
  write_bin(out_dir + "/solreffriction" + tag + ".bin", solreffriction.data(),
            sizeof(float) * solreffriction.size());
  write_bin(out_dir + "/solimp" + tag + ".bin", solimp.data(), sizeof(float) * solimp.size());
  write_bin(out_dir + "/adhesion" + tag + ".bin", adhesion.data(), sizeof(float) * adhesion.size());
  write_bin(out_dir + "/dim" + tag + ".bin", dim.data(), sizeof(int) * dim.size());
  write_bin(out_dir + "/geom" + tag + ".bin", geom.data(), sizeof(int) * geom.size());
  write_bin(out_dir + "/efc_address" + tag + ".bin", efc.data(), sizeof(int) * efc.size());
  write_bin(out_dir + "/worldid" + tag + ".bin", worldid.data(), sizeof(int) * worldid.size());
  write_bin(out_dir + "/type" + tag + ".bin", type.data(), sizeof(int) * type.size());
  write_bin(out_dir + "/geomcollisionid" + tag + ".bin", geomcollisionid.data(),
            sizeof(int) * geomcollisionid.size());
  write_bin(out_dir + "/nacon" + tag + ".bin", nacon_host.data(), sizeof(int));
  write_bin(out_dir + "/nccd" + tag + ".bin", nccd_host.data(), sizeof(int));
  cudaMemcpy(prism_count.data(), out.prism_count, sizeof(int) * worlds,
             cudaMemcpyDeviceToHost);
  write_bin(out_dir + "/prism_count" + tag + ".bin", prism_count.data(),
            sizeof(int) * worlds);
  write_bin(out_dir + "/overflow" + tag + ".bin", overflow_host.data(),
            sizeof(unsigned) * overflow_host.size());
  std::printf("%s kind=%s worlds=%d nacon=%d nccd=%d overflow=%u\n", case_dir.c_str(),
              kind.c_str(), worlds, nacon_host[0], nccd_host[0],
              worlds ? overflow_host[0] : 0u);
}

static void run_witness_case(const std::string& case_dir, const std::string& out_dir,
                             bool second_run) {
  auto meta = read_meta(case_dir + "/meta.txt");
  QaCase c{};
  c.nworld = meta_i32(meta, "nworld");
  c.ngeom = meta_i32(meta, "ngeom");
  c.naconmax = meta_i32(meta, "naconmax");
  c.naccdmax = meta_i32(meta, "naccdmax");
  c.efc_width = meta_i32(meta, "efc_width");
  c.g1 = meta_i32(meta, "g1");
  c.g2 = meta_i32(meta, "g2");
  c.type1 = meta_i32(meta, "type1");
  c.type2 = meta_i32(meta, "type2");
  c.kind = 3;
  c.gjk_iterations = meta_i32(meta, "gjk_iterations");
  c.epa_iterations = meta_i32(meta, "epa_iterations");
  c.tolerance = meta_f32(meta, "tolerance");
  c.margin = meta_f32(meta, "margin");
  c.gap = meta_f32(meta, "gap");
  c.max_horizon = meta.count("max_horizon") ? meta_i32(meta, "max_horizon") : 24;
  auto geom_xpos = read_bin<float>(case_dir + "/geom_xpos.bin");
  auto geom_xmat = read_bin<float>(case_dir + "/geom_xmat.bin");
  auto geom_size = read_bin<float>(case_dir + "/geom_size.bin");
  auto geom_margin = read_bin<float>(case_dir + "/geom_margin.bin");
  auto geom_gap = read_bin<float>(case_dir + "/geom_gap.bin");
  auto geom_rbound = read_bin<float>(case_dir + "/geom_rbound.bin");
  auto geom_aabb = read_bin<float>(case_dir + "/geom_aabb.bin");
  auto geom_type = read_bin<int>(case_dir + "/geom_type.bin");
  auto geom_dataid = read_bin<int>(case_dir + "/geom_dataid.bin");
  auto mesh_vertadr = read_bin<int>(case_dir + "/mesh_vertadr.bin");
  auto mesh_vertnum = read_bin<int>(case_dir + "/mesh_vertnum.bin");
  auto mesh_graphadr = read_bin<int>(case_dir + "/mesh_graphadr.bin");
  auto mesh_octadr = read_bin<int>(case_dir + "/mesh_octadr.bin");
  auto mesh_vert = read_bin<float>(case_dir + "/mesh_vert.bin");
  auto mesh_graph = read_bin<int>(case_dir + "/mesh_graph.bin");
  auto hfield_size = read_bin<float>(case_dir + "/hfield_size.bin");
  auto hfield_nrow = read_bin<int>(case_dir + "/hfield_nrow.bin");
  auto hfield_ncol = read_bin<int>(case_dir + "/hfield_ncol.bin");
  auto hfield_adr = read_bin<int>(case_dir + "/hfield_adr.bin");
  auto hfield_data = read_bin<float>(case_dir + "/hfield_data.bin");
  auto oct_aabb = read_bin<float>(case_dir + "/oct_aabb.bin");
  auto oct_child = read_bin<int>(case_dir + "/oct_child.bin");
  auto oct_coeff = read_bin<float>(case_dir + "/oct_coeff.bin");
  c.geom_xpos = upload(geom_xpos);
  c.geom_xmat = upload(geom_xmat);
  c.geom_size = upload(geom_size);
  c.geom_margin = upload(geom_margin);
  c.geom_gap = upload(geom_gap);
  c.geom_rbound = upload(geom_rbound);
  c.geom_aabb = upload(geom_aabb);
  c.geom_type = upload(geom_type);
  c.geom_dataid = upload(geom_dataid);
  c.mesh_vertadr = upload(mesh_vertadr);
  c.mesh_vertnum = upload(mesh_vertnum);
  c.mesh_graphadr = upload(mesh_graphadr);
  c.mesh_octadr = upload(mesh_octadr);
  c.mesh_vert = upload(mesh_vert);
  c.mesh_graph = upload(mesh_graph);
  c.hfield_size = upload(hfield_size);
  c.hfield_nrow = upload(hfield_nrow);
  c.hfield_ncol = upload(hfield_ncol);
  c.hfield_adr = upload(hfield_adr);
  c.hfield_data = upload(hfield_data);
  c.oct_aabb = upload(oct_aabb);
  c.oct_child = upload(oct_child);
  c.oct_coeff = upload(oct_coeff);
  QaCase* dc = (QaCase*)upload(std::vector<QaCase>{c});
  cudaMemcpy(dc, &c, sizeof(QaCase), cudaMemcpyHostToDevice);
  check_cuda("witness case upload");

  const int worlds = c.nworld;
  std::vector<float> dist(worlds, 0.0f), v1((size_t)worlds * 3, 0.0f), v2((size_t)worlds * 3, 0.0f);
  std::vector<int> ncontact(worlds, 0), index(worlds, 0);
  std::vector<unsigned> overflow((size_t)worlds, 0u);
  QaWitnessOut out{};
  out.dist = upload(dist);
  out.v1 = upload(v1);
  out.v2 = upload(v2);
  out.ncontact = upload(ncontact);
  out.index = upload(index);
  out.overflow = upload(overflow);
  QaWitnessOut* dout = (QaWitnessOut*)upload(std::vector<QaWitnessOut>{out});
  cudaMemset(out.overflow, 0, sizeof(unsigned) * worlds);
  qa_witness_kernel<<<worlds, 1>>>(dc, dout);
  check_cuda("witness kernel");
  cudaDeviceSynchronize();
  check_cuda("witness kernel sync");
  cudaMemcpy(dist.data(), out.dist, sizeof(float) * worlds, cudaMemcpyDeviceToHost);
  cudaMemcpy(v1.data(), out.v1, sizeof(float) * worlds * 3, cudaMemcpyDeviceToHost);
  cudaMemcpy(v2.data(), out.v2, sizeof(float) * worlds * 3, cudaMemcpyDeviceToHost);
  cudaMemcpy(ncontact.data(), out.ncontact, sizeof(int) * worlds, cudaMemcpyDeviceToHost);
  cudaMemcpy(index.data(), out.index, sizeof(int) * worlds, cudaMemcpyDeviceToHost);
  cudaMemcpy(overflow.data(), out.overflow, sizeof(unsigned) * worlds,
             cudaMemcpyDeviceToHost);
  check_cuda("read witness");
  const std::string tag = second_run ? "-run2" : "";
  write_bin(out_dir + "/dist" + tag + ".bin", dist.data(), sizeof(float) * worlds);
  write_bin(out_dir + "/x1" + tag + ".bin", v1.data(), sizeof(float) * worlds * 3);
  write_bin(out_dir + "/x2" + tag + ".bin", v2.data(), sizeof(float) * worlds * 3);
  write_bin(out_dir + "/ncontact" + tag + ".bin", ncontact.data(), sizeof(int) * worlds);
  write_bin(out_dir + "/index" + tag + ".bin", index.data(), sizeof(int) * worlds);
  write_bin(out_dir + "/overflow" + tag + ".bin", overflow.data(), sizeof(unsigned) * worlds);
  std::printf("%s kind=witness worlds=%d dist0=%.9g\n", case_dir.c_str(), worlds,
              worlds ? dist[0] : 0.0f);
}

int main(int argc, char** argv) {
  if (argc < 4) {
    std::printf("usage: qa_probe <cases_root> <out_root> <cases.txt> [run_tag]\n");
    return 2;
  }
  const std::string cases_root = argv[1];
  const std::string out_root = argv[2];
  const std::string list_path = argv[3];
  const bool second_run = argc > 4 && std::string(argv[4]) == "run2";
  std::ifstream list(list_path);
  if (!list) die("cannot open cases list");
  std::string name;
  int failures = 0;
  while (list >> name) {
    const std::string case_dir = cases_root + "/" + name;
    auto meta = read_meta(case_dir + "/meta.txt");
    const std::string kind = meta_str(meta, "kind");
    std::string out_dir = out_root + "/" + name;
    std::string command = "mkdir \"" + out_dir + "\" 2>nul";
    if (std::system(command.c_str()) != 0) {
      // 目录已存在时 mkdir 返回非零，忽略
    }
    cudaError_t error = cudaSuccess;
    if (kind == "witness") {
      run_witness_case(case_dir, out_dir, second_run);
      error = cudaGetLastError();
    } else {
      run_contact_case(case_dir, out_dir, second_run);
      error = cudaGetLastError();
    }
    if (error != cudaSuccess) {
      std::fprintf(stderr, "case %s failed: %s\n", name.c_str(), cudaGetErrorString(error));
      failures += 1;
    }
  }
  std::printf("qa_probe done: failures=%d\n", failures);
  return failures == 0 ? 0 : 1;
}
