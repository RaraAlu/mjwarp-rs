// G5 · ellipsoid-box 设备逻辑诊断 harness（**diagnostic-only**）
//
// 不是产品代码、不是验收路径、不替代 GPU 证据。用途：在**不占用 GPU**的前提下，
// 把 `src/physics/collision/cuda/*.cu` 当普通 C++ 执行，导出设备侧中间量，
// 与宿主 `gjk` 内部同一组中间量逐项对照，定位 `ellipsoid-box` witness 差异。
//
// shim 宏替换（仅本文件内，device 源零改动）：
//   `#define __device__`            → 空
//   `#define __forceinline__ inline`
//   `__fmaf_rn(a,b,c)`             → `std::fma(a,b,c)`（单次正确舍入，模拟 fma）
//   `atomicAdd/atomicOr`           → 单线程 CPU 实现（本路径不使用原子写接触）
//
// 编译（仓库根目录，需 MSVC；**不参与产品构建**）：
//   cmd /c "\"...\vcvars64.bat\" && cl /nologo /utf-8 /std:c++17 /EHsc /O2 \
//     /I src\physics\collision\cuda \
//     docs/development-evidence/g10-p5-gpu-kernels/gpu-window/device_logic_probe.cpp \
//     /Fe:target/g10-p5-g5/device_logic_probe.exe /Fo:target/g10-p5-g5/device_logic_probe.obj"
//
// 运行（输入为 prepare_cases.py 产出的 case 目录）：
//   target/g10-p5-g5/device_logic_probe.exe \
//     docs/development-evidence/g10-p5-gpu-kernels/gpu-window/cases/ellipsoid-box
//
// 输出：每 world 一行 `world=... gjk_dist=... epa_dist=... face_idx=... lower=... normal=...`，
// 以及 `witness` 行（`x1`/`x2`/`pos`/frame）、`face` 行（三顶点索引/坐标/权重）。
#define __device__
#define __forceinline__ inline

#include <cmath>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <fstream>
#include <map>
#include <string>
#include <vector>

inline float __fmaf_rn(float a, float b, float c) { return std::fma(a, b, c); }
inline int atomicAdd(int* pointer, int value) {
  int old = *pointer;
  *pointer = old + value;
  return old;
}
inline unsigned atomicOr(unsigned* pointer, unsigned value) {
  unsigned old = *pointer;
  *pointer = old | value;
  return old;
}

#include "contact.cu"

static std::map<std::string, std::string> read_meta(const std::string& directory) {
  std::map<std::string, std::string> meta;
  std::ifstream input(directory + "/meta.txt");
  std::string line;
  while (std::getline(input, line)) {
    if (line.empty()) continue;
    const auto space = line.find(' ');
    meta[line.substr(0, space)] = line.substr(space + 1);
  }
  return meta;
}

template <typename T>
static std::vector<T> read_bin(const std::string& directory, const std::string& name) {
  std::ifstream input(directory + "/" + name + ".bin", std::ios::binary);
  std::vector<char> bytes((std::istreambuf_iterator<char>(input)),
                          std::istreambuf_iterator<char>());
  std::vector<T> typed(bytes.size() / sizeof(T));
  if (!bytes.empty()) std::memcpy(typed.data(), bytes.data(), typed.size() * sizeof(T));
  return typed;
}

struct LocalPolytope {
  v3 vert[2 * (10 + 2 * 64)];
  int vert_index[2 * (10 + 2 * 64)];
  unsigned face[6 + 5 * 64];
  v3 face_pr[6 + 5 * 64];
  float face_norm2[6 + 5 * 64];
  int horizon[24];
  d_polytope pt;
};

static void init_polytope(LocalPolytope* local) {
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
  local->pt.max_face = 6 + 5 * 64;
  local->pt.max_horizon = 24;
}

template <typename T>
static void copy4(T target[4], const T source[4]) {
  for (int index = 0; index < 4; ++index) target[index] = source[index];
}

int main(int argc, char** argv) {
  if (argc < 2) return 2;
  const std::string directory = argv[1];
  auto meta = read_meta(directory);
  auto i32 = [&](const char* key) { return std::atoi(meta[key].c_str()); };
  auto f32 = [&](const char* key) { return std::strtof(meta[key].c_str(), nullptr); };

  const int nworld = i32("nworld");
  const int ngeom = i32("ngeom");
  const int g1 = i32("g1");
  const int g2 = i32("g2");
  const int type1 = i32("type1");
  const int type2 = i32("type2");
  const float tolerance = f32("tolerance");
  const float margin = f32("margin");
  const float gap = f32("gap");
  const int gjk_iterations = i32("gjk_iterations");
  const int epa_iterations = i32("epa_iterations");

  const auto xpos = read_bin<float>(directory, "geom_xpos");
  const auto xmat = read_bin<float>(directory, "geom_xmat");
  const auto size = read_bin<float>(directory, "geom_size");
  const auto geom_margin = read_bin<float>(directory, "geom_margin");

  for (int world = 0; world < nworld; ++world) {
    auto build = [&](int index) {
      d_geom geom{};
      const int base = (world * ngeom + index) * 3;
      geom.pos = v3_make(xpos[base], xpos[base + 1], xpos[base + 2]);
      const int rot_base = (world * ngeom + index) * 9;
      for (int k = 0; k < 9; ++k) geom.rot[k] = xmat[rot_base + k];
      geom.size = v3_make(size[index * 3], size[index * 3 + 1], size[index * 3 + 2]);
      geom.margin = 0.0f;
      geom.vertadr = -1;
      geom.vertnum = -1;
      geom.graphadr = -1;
      geom.index = -1;
      (void)geom_margin;
      return geom;
    };
    d_geom geom1 = build(g1);
    d_geom geom2 = build(g2);
    geom1.margin = margin;
    geom2.margin = margin;

    const v3 origin = geom1.pos;
    geom1.pos = v3_make(0.0f, 0.0f, 0.0f);
    geom2.pos = v3_sub(geom2.pos, origin);

    d_gjk_phase phase;
    g5_gjk_phase(tolerance, gap, gjk_iterations, geom1, geom2, type1, type2, geom1.pos, geom2.pos,
                 &phase);
    std::printf("world=%d phase needs_epa=%d gjk_dist=%.9g ncontact=%d dim=%d\n", world,
                (int)phase.needs_epa, phase.dist, phase.ncontact, phase.result.dim);

    // 复刻 g5_epa_phase 的 polytope 构造，但直接调用 g5_epa 以拿到 face_idx。
    LocalPolytope local;
    init_polytope(&local);
    d_gjk_result result = phase.result;
    d_gjk_result next = g5_gjk_result_zero();
    const int epa_cap = epa_iterations < 64 ? epa_iterations : 64;
    (void)epa_cap;
    if (result.dim == 2) {
      g5_polytope2(&local.pt, result.simplex, result.simplex1, result.simplex2,
                   result.simplex_index1, result.simplex_index2, phase.geom1, phase.geom2, type1,
                   type2, &next);
      if (local.pt.status == -1) {
        copy4(result.simplex, next.simplex);
        copy4(result.simplex1, next.simplex1);
        copy4(result.simplex2, next.simplex2);
        copy4(result.simplex_index1, next.simplex_index1);
        copy4(result.simplex_index2, next.simplex_index2);
        result.dim = 3;
      }
    } else if (result.dim == 4) {
      g5_polytope4(&local.pt, result.simplex, result.simplex1, result.simplex2, result.simplex_index1,
                   result.simplex_index2, &next);
      if (local.pt.status == -1) {
        copy4(result.simplex, next.simplex);
        copy4(result.simplex1, next.simplex1);
        copy4(result.simplex2, next.simplex2);
        copy4(result.simplex_index1, next.simplex_index1);
        copy4(result.simplex_index2, next.simplex_index2);
        result.dim = 3;
      }
    }
    if (result.dim == 3) {
      g5_polytope3(&local.pt, result.dist, result.simplex, result.simplex1, result.simplex2,
                   result.simplex_index1, result.simplex_index2, phase.geom1, phase.geom2, type1,
                   type2);
    }
    if (local.pt.status != 0) {
      std::printf("world=%d epa skipped status=%d\n", world, local.pt.status);
      continue;
    }
    const bool discrete = g5_is_discrete(type1, type2, phase.geom1.margin, phase.geom2.margin);
    const d_epa_result epa =
        g5_epa(&local.pt, tolerance, epa_iterations, phase.geom1, phase.geom2, type1, type2,
               discrete);
    std::printf("world=%d epa face_idx=%d epa_dist=%.9g x1=(%.9g,%.9g,%.9g) "
                "x2=(%.9g,%.9g,%.9g) overflow=%u\n",
                world, epa.face_idx, epa.dist, epa.x1.x, epa.x1.y, epa.x1.z, epa.x2.x, epa.x2.y,
                epa.x2.z, epa.overflow);
    if (epa.face_idx >= 0) {
      const int face_index = epa.face_idx;
      const float lower = std::sqrt(local.pt.face_norm2[face_index]);
      const v3 pr = local.pt.face_pr[face_index];
      const v3 normal = v3_make(pr.x / lower, pr.y / lower, pr.z / lower);
      int face[3];
      g5_get_face_verts(local.pt.face[face_index], face);
      const v3 v1 = v3_sub(local.pt.vert[2 * face[0]], local.pt.vert[2 * face[0] + 1]);
      const v3 v2 = v3_sub(local.pt.vert[2 * face[1]], local.pt.vert[2 * face[1] + 1]);
      const v3 v3v = v3_sub(local.pt.vert[2 * face[2]], local.pt.vert[2 * face[2] + 1]);
      float weights[3];
      g5_tri_affine_coord(v1, v2, v3v, local.pt.face_pr[face_index], weights);
      std::printf("world=%d face lower=%.9g normal=(%.9g,%.9g,%.9g) verts=(%d,%d,%d) "
                  "w=(%.9g,%.9g,%.9g) pr=(%.9g,%.9g,%.9g)\n",
                  world, lower, normal.x, normal.y, normal.z, face[0], face[1], face[2], weights[0],
                  weights[1], weights[2], pr.x, pr.y, pr.z);
      std::printf("world=%d face_v1=(%.9g,%.9g,%.9g) face_v2=(%.9g,%.9g,%.9g) "
                  "face_v3=(%.9g,%.9g,%.9g)\n",
                  world, local.pt.vert[2 * face[0]].x, local.pt.vert[2 * face[0]].y,
                  local.pt.vert[2 * face[0]].z, local.pt.vert[2 * face[1]].x,
                  local.pt.vert[2 * face[1]].y, local.pt.vert[2 * face[1]].z,
                  local.pt.vert[2 * face[2]].x, local.pt.vert[2 * face[2]].y,
                  local.pt.vert[2 * face[2]].z);
    }
    const float final_dist = epa.dist + margin;
    const v3 pos = v3_add(v3_scale(v3_add(epa.x1, epa.x2), 0.5f), origin);
    std::printf("world=%d witness final_dist=%.9g pos=(%.9g,%.9g,%.9g)\n", world, final_dist,
                pos.x, pos.y, pos.z);
  }
  return 0;
}
