// G10@P5 B1 plane_convex 设备实现（pinned collision_primitive.py L52-278 平移）。
// 唯一入口：g5_plane_convex（穷举 + graph hillclimb + 4 点去重接触收集）。
#include "common.cu"

// ---------------------------------------------------------------------------------------------
// g5_plane_convex：pinned `plane_convex` L52-278 的完整平移版（穷举 + graph hillclimb +
// 顶点访问最终形态：mesh_vert 全体 + (vertadr + idx)；graph = mesh_graph + d_geom.graphadr 偏移。
// ---------------------------------------------------------------------------------------------

// graph hillclimb 通用核：返回局部 index（供 a/b/c/d 复用），score 返回 -HUGE 表示不候选。
template <typename Score>
__device__ __forceinline__ int g5_plane_graph_hillclimb(const int* graph, int graphadr, int start,
                                                        Score score) {
  const int numvert = graph[graphadr];
  const int vert_edgeadr = graphadr + 2;
  const int vert_globalid = graphadr + 2 + numvert;
  const int edge_localid = graphadr + 2 + 2 * numvert;
  int imax = start;
  // pinned L147-165/L169-183 等：每阶段维护 running best（初值 -HUGE），严格 `> best` 才更新。
  float best = -G5_PLANE_HUGE_VAL;
  for (int guard = 0; guard < (1 << 20); ++guard) {
    const int prev = imax;
    int i = graph[vert_edgeadr + imax];
    while (graph[edge_localid + i] >= 0) {
      const int subidx = graph[edge_localid + i];
      const float value = score(graph[vert_globalid + subidx]);
      if (value > best) {
        best = value;
        imax = subidx;
      }
      i += 1;
    }
    if (imax == prev) break;
  }
  return imax;
}

// 完整版：穷举分支（vertnum < 10 或 graphadr == -1）与 graph 分支都返回最多 4 条唯一索引接触。
__device__ __forceinline__ int g5_plane_convex(v3 plane_normal, v3 plane_pos,
                                                    const d_geom& convex, const float* vert,
                                                    const int* graph, g5_plane_contact out[4]) {
  const int vertadr = convex.vertadr;
  const v3 plane_pos_local = transpose_mul_mat3_vec(convex.rot, v3_sub(plane_pos, convex.pos));
  const v3 n = transpose_mul_mat3_vec(convex.rot, plane_normal);
  const auto vert_at = [&](int idx) -> v3 {
    const int base = (vertadr + idx) * 3;
    return v3_make(vert[base], vert[base + 1], vert[base + 2]);
  };
  const auto support_at = [&](int idx) -> float {
    return v3_dot(v3_sub(plane_pos_local, vert_at(idx)), n);
  };
  for (int i = 0; i < 4; ++i) {
    out[i].dist = G5_PLANE_MAXVAL;
    out[i].pos = v3_make(0.0f, 0.0f, 0.0f);
    out[i].normal = plane_normal;
  }
  int indices[4] = {-1, -1, -1, -1};
  v3 a = v3_make(0.0f, 0.0f, 0.0f);
  v3 b = v3_make(0.0f, 0.0f, 0.0f);
  v3 c = v3_make(0.0f, 0.0f, 0.0f);
  float threshold = 0.0f;

  if (convex.graphadr == -1 || convex.vertnum < 10) {
    // 穷举分支（pinned L79-138）。
    const int count = convex.vertnum;
    float max_support = -G5_PLANE_HUGE_VAL;
    for (int i = 0; i < count; ++i) {
      const float support = support_at(i);
      if (support > max_support) { max_support = support; indices[0] = i; a = vert_at(i); }
    }
    if (max_support < 0.0f) return 0;
    threshold = max_support - G5_PLANE_CONVEX_THRESHOLD;
    float b_dist = -G5_PLANE_HUGE_VAL;
    for (int i = 0; i < count; ++i) {
      const float mask = support_at(i) > threshold ? 0.0f : -G5_PLANE_HUGE_VAL;
      const v3 d = v3_sub(a, vert_at(i));
      const float dist = v3_dot(d, d) + mask;
      if (dist > b_dist) { b_dist = dist; indices[1] = i; b = vert_at(i); }
    }
    const v3 ab = v3_cross(n, v3_sub(a, b));
    float c_dist = -G5_PLANE_HUGE_VAL;
    for (int i = 0; i < count; ++i) {
      const float mask = support_at(i) > threshold ? 0.0f : -G5_PLANE_HUGE_VAL;
      const float dist = fabsf(v3_dot(v3_sub(a, vert_at(i)), ab)) + mask;
      if (dist > c_dist) { c_dist = dist; indices[2] = i; c = vert_at(i); }
    }
    const v3 ac = v3_cross(n, v3_sub(a, c));
    const v3 bc = v3_cross(n, v3_sub(b, c));
    float d_dist = -G5_PLANE_HUGE_VAL;
    for (int i = 0; i < count; ++i) {
      const float mask = support_at(i) > threshold ? 0.0f : -G5_PLANE_HUGE_VAL;
      const float dist = fabsf(v3_dot(v3_sub(a, vert_at(i)), ac)) +
                         fabsf(v3_dot(v3_sub(b, vert_at(i)), bc)) + mask;
      if (dist > d_dist) { d_dist = dist; indices[3] = i; }
    }
  } else {
    // graph hillclimb 分支（pinned L141-255）。
    const int graphadr = convex.graphadr;
    const int vert_globalid_local = graphadr + 2 + graph[graphadr];
    int imax = 0;
    float max_support = -G5_PLANE_HUGE_VAL;
    const auto support_score = [&](int idx) -> float {
      const float support = support_at(idx);
      if (support > max_support) max_support = support;
      return support;
    };
    imax = g5_plane_graph_hillclimb(graph, graphadr, 0, support_score);
    threshold = fmaxf(0.0f, max_support - G5_PLANE_CONVEX_THRESHOLD);
    const auto a_score = [&](int idx) -> float {
      const float support = support_at(idx);
      return support > threshold ? support : -G5_PLANE_HUGE_VAL;
    };
    imax = g5_plane_graph_hillclimb(graph, graphadr, imax, a_score);
    indices[0] = graph[vert_globalid_local + imax];
    a = vert_at(indices[0]);
    const auto b_score = [&](int idx) -> float {
      const float mask = support_at(idx) > threshold ? 0.0f : -G5_PLANE_HUGE_VAL;
      const v3 d = v3_sub(a, vert_at(idx));
      return v3_dot(d, d) + mask;
    };
    imax = g5_plane_graph_hillclimb(graph, graphadr, imax, b_score);
    indices[1] = graph[vert_globalid_local + imax];
    b = vert_at(indices[1]);
    const v3 ab = v3_cross(n, v3_sub(a, b));
    const auto c_score = [&](int idx) -> float {
      const float mask = support_at(idx) > threshold ? 0.0f : -G5_PLANE_HUGE_VAL;
      return fabsf(v3_dot(v3_sub(a, vert_at(idx)), ab)) + mask;
    };
    imax = g5_plane_graph_hillclimb(graph, graphadr, imax, c_score);
    indices[2] = graph[vert_globalid_local + imax];
    c = vert_at(indices[2]);
    const v3 ac = v3_cross(n, v3_sub(a, c));
    const v3 bc = v3_cross(n, v3_sub(b, c));
    const auto d_score = [&](int idx) -> float {
      const float mask = support_at(idx) > threshold ? 0.0f : -G5_PLANE_HUGE_VAL;
      return fabsf(v3_dot(v3_sub(a, vert_at(idx)), ac)) +
             fabsf(v3_dot(v3_sub(b, vert_at(idx)), bc)) + mask;
    };
    imax = g5_plane_graph_hillclimb(graph, graphadr, imax, d_score);
    indices[3] = graph[vert_globalid_local + imax];
  }
  // 4 点去重接触收集（pinned L257-277）。
  int contact_count = 0;
  for (int i = 3; i >= 0; --i) {
    const int idx = indices[i];
    int count = 0;
    for (int j = 0; j <= i; ++j) {
      if (indices[j] == idx) count += 1;
    }
    if (count == 1) {
      const v3 local = vert_at(idx);
      const v3 world = v3_add(convex.pos, mul_mat3_vec(convex.rot, local));
      const float dist = -v3_dot(v3_sub(plane_pos_local, local), n);
      out[contact_count].dist = dist;
      out[contact_count].pos = v3_sub(world, v3_scale(plane_normal, 0.5f * dist));
      out[contact_count].normal = plane_normal;
      contact_count += 1;
    }
  }
  return contact_count;
}

