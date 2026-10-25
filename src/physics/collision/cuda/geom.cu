// G10@P5 G5 设备侧几何视图（与 host `gjk::Geom`/`MeshView` 字段对应）。
// Adapted from frozen MJWarp collision_core.py, revision 71da24d. Apache-2.0.
#pragma once
#include "common.cu"

struct d_mesh {
  // mesh_vert：全局顶点数组，按 [nmeshvert][3] 展平。
  const float* vert;
  // mesh_graph：全局图数组。
  const int* graph;
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
  d_mesh mesh;
};

struct d_support_point {
  v3 point;
  // mesh 支撑热启动缓存索引（-1 表示无缓存）。
  int cached_index;
  // box 顶点编码 / cylinder 顶底 / hfield 边标记。
  int vertex_index;
};
