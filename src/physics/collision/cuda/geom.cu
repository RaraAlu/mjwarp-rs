// G10@P5 G5 设备侧几何视图（与 host `gjk::Geom`/`MeshView` 字段对应）。
// Adapted from frozen MJWarp collision_core.py, revision 71da24d. Apache-2.0.
#pragma once
#include "common.cu"

struct d_mesh {
  // mesh_vert：全局顶点数组，按 [nmeshvert][3] 展平。
  const float* vert;
  // mesh_graph：全局图数组。
  const int* graph;
  // M2：mesh polygon sidecar（与 `product.cu::MjwarpMeshPolyArgs` 同序、同形状语义：
  // polynum/polyadr 逐 mesh；polynormal/polyvertadr/polyvertnum 逐 polygon；
  // polymapadr/polymapnum 逐 vertex；polyvert/polymap 为扁平索引）。sidecar 未提供时
  // 9 个指针全为 nullptr（mesh 多接触不可用 → `g5_multicontact` 回退单接触）。
  const int* polynum;
  const int* polyadr;
  const float* polynormal;
  const int* polyvertadr;
  const int* polyvertnum;
  const int* polyvert;
  const int* polymapadr;
  const int* polymapnum;
  const int* polymap;
};

struct d_geom {
  v3 pos;
  // 行优先 3×3 旋转矩阵。
  float rot[9];
  v3 size;
  float margin;
  // hfield/flex 六顶点（hfield 由调用方在图元帧内预置）。
  float polyvert[18];
  int vertadr;
  int vertnum;
  int graphadr;
  int index;
  // M2：mesh 的 dataid（非 mesh 或非 mesh data → -1）。
  int dataid;
  // M2：`mesh_polyadr[dataid]`（逐 mesh 的首个 polygon 地址）；非 mesh / sidecar 缺失 → -1。
  int mesh_polyadr;
  d_mesh mesh;
};

// M2：把 polygon sidecar 指针清空（无 sidecar 时的写死语义）。
__device__ __forceinline__ void g5_mesh_clear_poly(d_mesh* mesh) {
  mesh->polynum = nullptr;
  mesh->polyadr = nullptr;
  mesh->polynormal = nullptr;
  mesh->polyvertadr = nullptr;
  mesh->polyvertnum = nullptr;
  mesh->polyvert = nullptr;
  mesh->polymapadr = nullptr;
  mesh->polymapnum = nullptr;
  mesh->polymap = nullptr;
}

// M2：读 mesh 顶点（局部索引 `index` 已含 `vertadr` 偏移；全局 mesh_vert 为 [nmeshvert][3]）。
__device__ __forceinline__ v3 g5_mesh_vertex(const float* vert, int index) {
  return v3_make(vert[index * 3], vert[index * 3 + 1], vert[index * 3 + 2]);
}

struct d_support_point {
  v3 point;
  // mesh 支撑热启动缓存索引（-1 表示无缓存）。
  int cached_index;
  // box 顶点编码 / cylinder 顶底 / hfield 边标记。
  int vertex_index;
};
