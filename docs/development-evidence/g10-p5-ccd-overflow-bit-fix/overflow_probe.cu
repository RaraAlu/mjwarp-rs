// G10@P5 修正片 · 强制 horizon 溢出位探针（authoring-only，不进产品构建）。
//
// 目的：验证设备 `g5_ccd`（`src/physics/collision/cuda/ccd.cu`）在 EPA 返回
// `FLOAT_MAX`（horizon 容量不足，`g5_add_edge` 返回 -1）时**不再**额外置
// `OverflowType.CCD(1<<4)`，只保留 `EPA_HORIZON(1<<8)`，与 pinned 上游
// `collision_gjk.ccd()` 位级一致。
//
// 做法：与 G5 `witness_probe.cu` 同一调用口径（`eval_ccd_write_contact` 的 ccd 段：
// margin 覆盖 + 原点平移 + cutoff=gap），但 polytope 的 `max_horizon` 由命令行给出；
// 用 `max_horizon=2` 确定性触发 horizon 溢出（上游对应做法：把 `ccd()` 的 `horizon`
// 数组长度设为 2，其 `shape[0]` 即 `Polytope.max_horizon`）。输入为 G5 冻结窗口的
// case bins（只读），输出写到独立 out_root，不改冻结证据目录。
//
// 构建（不占 GPU）：见 `build_probes.ps1`（nvcc 12.8.1，`-O3 --fmad=true -arch=sm_89`）。
// 用法：overflow_probe <case_dir> <max_horizon> <out_root> [repeats]
// 输出：<out_root>/<case>/run<N>/{dist,ncontact,x1,x2,multiccd_idx,overflow}.bin
//       + <out_root>/report-<case>.txt

#include <cuda_runtime.h>
#include <chrono>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <fstream>
#include <map>
#include <string>
#include <vector>

#include "ccd.cu"

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

template <typename T>
T* to_device(const std::vector<T>& host) {
  T* device = nullptr;
  const size_t bytes = host.size() * sizeof(T);
  PROBE_CHECK(cudaMalloc(&device, bytes == 0 ? sizeof(T) : bytes));
  if (!host.empty()) PROBE_CHECK(cudaMemcpy(device, host.data(), bytes, cudaMemcpyHostToDevice));
  return device;
}

constexpr int kEpaCap = 64;
constexpr int kHorizonArray = 24;  // 数组上界；实际容量由 max_horizon 决定。

struct LocalPolytope {
  v3 vert[2 * (10 + 2 * kEpaCap)];
  int vert_index[2 * (10 + 2 * kEpaCap)];
  unsigned face[6 + 5 * kEpaCap];
  v3 face_pr[6 + 5 * kEpaCap];
  float face_norm2[6 + 5 * kEpaCap];
  int horizon[kHorizonArray];
  d_polytope pt;
};

__device__ __forceinline__ void init_polytope(LocalPolytope* local, int max_horizon) {
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
  local->pt.max_horizon = max_horizon;
}

struct ProbeArgs {
  const float* geom_xpos;
  const float* geom_xmat;
  const float* geom_size;
  const float* geom_margin;
  const int* geom_type;
  const int* geom_dataid;
  const int* mesh_vertadr;
  const int* mesh_vertnum;
  const int* mesh_graphadr;
  const float* mesh_vert;
  const int* mesh_graph;

  int nworld;
  int ngeom;
  int g1;
  int g2;
  int type1;
  int type2;
  float tolerance;
  float margin;
  float gap;
  int gjk_iterations;
  int epa_iterations;
  int max_horizon;

  float* dist;
  int* ncontact;
  v3* x1;
  v3* x2;
  int* multiccd_idx;
  unsigned* overflow;
};

__device__ __forceinline__ d_geom build_geom(const ProbeArgs& a, int world, int geom_index) {
  d_geom geom;
  const int base = (world * a.ngeom + geom_index) * 3;
  geom.pos = v3_make(a.geom_xpos[base], a.geom_xpos[base + 1], a.geom_xpos[base + 2]);
  const int rot_base = (world * a.ngeom + geom_index) * 9;
  for (int index = 0; index < 9; ++index) geom.rot[index] = a.geom_xmat[rot_base + index];
  geom.size = v3_make(a.geom_size[geom_index * 3], a.geom_size[geom_index * 3 + 1],
                      a.geom_size[geom_index * 3 + 2]);
  geom.margin = a.geom_margin[geom_index];
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

// 上游 `eval_ccd_write_contact` 的 ccd 段：margin 覆盖 + 原点平移到 geom1 + cutoff=gap。
__global__ void kernel_overflow(ProbeArgs a) {
  const int world = blockIdx.x;
  if (world >= a.nworld) return;
  LocalPolytope local;
  init_polytope(&local, a.max_horizon);

  d_geom geom1 = build_geom(a, world, a.g1);
  d_geom geom2 = build_geom(a, world, a.g2);
  geom1.margin = a.margin;
  geom2.margin = a.margin;

  const v3 origin = geom1.pos;
  geom1.pos = v3_make(0.0f, 0.0f, 0.0f);
  geom2.pos = v3_sub(geom2.pos, origin);

  d_ccd_options options{a.tolerance, a.gap, a.gjk_iterations, a.epa_iterations};
  d_ccd_result result;
  g5_ccd(geom1, geom2, a.type1, a.type2, geom1.pos, geom2.pos, options, &local.pt, &result);

  a.dist[world] = result.dist;
  a.ncontact[world] = result.ncontact;
  a.x1[world] = result.x1;
  a.x2[world] = result.x2;
  a.multiccd_idx[world] = result.multiccd_idx;
  a.overflow[world] = result.overflow;
}

struct RunOutput {
  std::vector<float> dist;
  std::vector<int> ncontact;
  std::vector<float> x1;
  std::vector<float> x2;
  std::vector<int> multiccd_idx;
  std::vector<unsigned> overflow;
  double milliseconds = 0.0;
};

void make_directory(const std::string& path) {
  std::system(("mkdir \"" + path + "\" 2>nul").c_str());
}

}  // namespace

int main(int argc, char** argv) {
  if (argc < 4) {
    std::fprintf(stderr, "usage: overflow_probe <case_dir> <max_horizon> <out_root> [repeats]\n");
    return 2;
  }
  const std::string directory = argv[1];
  const int max_horizon = std::atoi(argv[2]);
  const std::string out_root = argv[3];
  const int repeats = argc > 4 ? std::atoi(argv[4]) : 2;
  if (max_horizon < 1 || max_horizon > kHorizonArray) {
    std::fprintf(stderr, "max_horizon out of range: %d\n", max_horizon);
    return 2;
  }
  const CaseMeta meta = read_meta(directory);

  ProbeArgs args{};
  args.geom_xpos = to_device(load_bin<float>(directory, "geom_xpos"));
  args.geom_xmat = to_device(load_bin<float>(directory, "geom_xmat"));
  args.geom_size = to_device(load_bin<float>(directory, "geom_size"));
  args.geom_margin = to_device(load_bin<float>(directory, "geom_margin"));
  args.geom_type = to_device(load_bin<int>(directory, "geom_type"));
  args.geom_dataid = to_device(load_bin<int>(directory, "geom_dataid"));
  args.mesh_vertadr = to_device(load_bin<int>(directory, "mesh_vertadr"));
  args.mesh_vertnum = to_device(load_bin<int>(directory, "mesh_vertnum"));
  args.mesh_graphadr = to_device(load_bin<int>(directory, "mesh_graphadr"));
  args.mesh_vert = to_device(load_bin<float>(directory, "mesh_vert"));
  args.mesh_graph = to_device(load_bin<int>(directory, "mesh_graph"));

  args.nworld = meta.i32("nworld");
  args.ngeom = meta.i32("ngeom");
  args.g1 = meta.i32("g1");
  args.g2 = meta.i32("g2");
  args.type1 = meta.i32("type1");
  args.type2 = meta.i32("type2");
  args.tolerance = meta.f32("tolerance");
  args.margin = meta.f32("margin");
  args.gap = meta.f32("gap");
  args.gjk_iterations = meta.i32("gjk_iterations");
  args.epa_iterations = meta.i32("epa_iterations");
  args.max_horizon = max_horizon;

  const int nworld = args.nworld;
  float* d_dist = nullptr;
  int* d_ncon = nullptr;
  v3* d_x1 = nullptr;
  v3* d_x2 = nullptr;
  int* d_idx = nullptr;
  unsigned* d_overflow = nullptr;
  PROBE_CHECK(cudaMalloc(&d_dist, nworld * sizeof(float)));
  PROBE_CHECK(cudaMalloc(&d_ncon, nworld * sizeof(int)));
  PROBE_CHECK(cudaMalloc(&d_x1, nworld * sizeof(v3)));
  PROBE_CHECK(cudaMalloc(&d_x2, nworld * sizeof(v3)));
  PROBE_CHECK(cudaMalloc(&d_idx, nworld * sizeof(int)));
  PROBE_CHECK(cudaMalloc(&d_overflow, nworld * sizeof(unsigned)));
  args.dist = d_dist;
  args.ncontact = d_ncon;
  args.x1 = d_x1;
  args.x2 = d_x2;
  args.multiccd_idx = d_idx;
  args.overflow = d_overflow;

  cudaDeviceProp properties{};
  PROBE_CHECK(cudaGetDeviceProperties(&properties, 0));
  size_t free_bytes = 0;
  size_t total_bytes = 0;
  PROBE_CHECK(cudaMemGetInfo(&free_bytes, &total_bytes));
  std::printf("device=%s sm=%d.%d cudart=%d free=%zuMiB total=%zuMiB\n", properties.name,
              properties.major, properties.minor, CUDART_VERSION, free_bytes >> 20,
              total_bytes >> 20);

  std::vector<RunOutput> runs;
  for (int repeat = 0; repeat < repeats; ++repeat) {
    PROBE_CHECK(cudaMemset(d_overflow, 0, nworld * sizeof(unsigned)));
    const auto started = std::chrono::steady_clock::now();
    kernel_overflow<<<nworld, 1>>>(args);
    PROBE_CHECK(cudaGetLastError());
    PROBE_CHECK(cudaDeviceSynchronize());
    const auto elapsed =
        std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - started)
            .count();

    RunOutput run;
    run.milliseconds = elapsed;
    run.dist.resize(nworld);
    run.ncontact.resize(nworld);
    run.x1.resize(nworld * 3);
    run.x2.resize(nworld * 3);
    run.multiccd_idx.resize(nworld);
    run.overflow.resize(nworld);
    PROBE_CHECK(cudaMemcpy(run.dist.data(), d_dist, nworld * sizeof(float), cudaMemcpyDeviceToHost));
    PROBE_CHECK(cudaMemcpy(run.ncontact.data(), d_ncon, nworld * sizeof(int), cudaMemcpyDeviceToHost));
    PROBE_CHECK(cudaMemcpy(run.x1.data(), d_x1, nworld * sizeof(v3), cudaMemcpyDeviceToHost));
    PROBE_CHECK(cudaMemcpy(run.x2.data(), d_x2, nworld * sizeof(v3), cudaMemcpyDeviceToHost));
    PROBE_CHECK(cudaMemcpy(run.multiccd_idx.data(), d_idx, nworld * sizeof(int),
                           cudaMemcpyDeviceToHost));
    PROBE_CHECK(cudaMemcpy(run.overflow.data(), d_overflow, nworld * sizeof(unsigned),
                           cudaMemcpyDeviceToHost));
    runs.push_back(run);
  }

  const bool raw_equal = runs.size() < 2 || [&]() {
    const RunOutput& a = runs[0];
    const RunOutput& b = runs[1];
    return a.dist == b.dist && a.ncontact == b.ncontact && a.x1 == b.x1 && a.x2 == b.x2 &&
           a.multiccd_idx == b.multiccd_idx && a.overflow == b.overflow;
  }();

  const std::string case_dir_out = out_root + "/" + meta.text("case");
  make_directory(out_root);
  make_directory(case_dir_out);
  for (size_t repeat = 0; repeat < runs.size(); ++repeat) {
    const std::string out_dir = case_dir_out + "/run" + std::to_string(repeat + 1);
    make_directory(out_dir);
    write_bin(out_dir, "dist", runs[repeat].dist);
    write_bin(out_dir, "ncontact", runs[repeat].ncontact);
    write_bin(out_dir, "x1", runs[repeat].x1);
    write_bin(out_dir, "x2", runs[repeat].x2);
    write_bin(out_dir, "multiccd_idx", runs[repeat].multiccd_idx);
    write_bin(out_dir, "overflow", runs[repeat].overflow);
  }

  std::printf("case=%s max_horizon=%d nworld=%d raw_equal=%s overflow=",
              meta.text("case").c_str(), max_horizon, nworld, raw_equal ? "true" : "false");
  for (unsigned value : runs[0].overflow) std::printf("0x%x,", value);
  std::printf(" times=");
  for (const RunOutput& run : runs) std::printf("%.3fms,", run.milliseconds);
  std::printf("\n");
  for (int world = 0; world < nworld; ++world) {
    std::printf("world=%d dist=%.9g ncontact=%d x1=(%.9g,%.9g,%.9g) x2=(%.9g,%.9g,%.9g) "
                "idx=%d overflow=0x%x\n",
                world, runs[0].dist[world], runs[0].ncontact[world], runs[0].x1[world * 3],
                runs[0].x1[world * 3 + 1], runs[0].x1[world * 3 + 2], runs[0].x2[world * 3],
                runs[0].x2[world * 3 + 1], runs[0].x2[world * 3 + 2], runs[0].multiccd_idx[world],
                runs[0].overflow[world]);
  }

  std::ofstream report(case_dir_out + "/report.txt");
  report << "case " << meta.text("case") << "\n";
  report << "max_horizon " << max_horizon << "\n";
  report << "device " << properties.name << "\n";
  report << "compute_capability " << properties.major << "." << properties.minor << "\n";
  report << "cudart " << CUDART_VERSION << "\n";
  report << "nworld " << nworld << "\n";
  report << "pair " << args.g1 << " " << args.g2 << "\n";
  report << "tolerance " << args.tolerance << "\n";
  report << "margin " << args.margin << "\n";
  report << "cutoff(gap) " << args.gap << "\n";
  report << "gjk_iterations " << args.gjk_iterations << "\n";
  report << "epa_iterations " << args.epa_iterations << "\n";
  report << "two_runs_raw_bitwise_identical " << (raw_equal ? "true" : "false") << "\n";
  for (int world = 0; world < nworld; ++world) {
    report << "world" << world << "_dist " << runs[0].dist[world] << "\n";
    report << "world" << world << "_ncontact " << runs[0].ncontact[world] << "\n";
    report << "world" << world << "_idx " << runs[0].multiccd_idx[world] << "\n";
    report << "world" << world << "_overflow " << runs[0].overflow[world] << "\n";
  }
  report << "free_mib_before " << (free_bytes >> 20) << "\n";
  size_t free_after = 0;
  size_t total_after = 0;
  PROBE_CHECK(cudaMemGetInfo(&free_after, &total_after));
  report << "free_mib_after " << (free_after >> 20) << "\n";
  return 0;
}
