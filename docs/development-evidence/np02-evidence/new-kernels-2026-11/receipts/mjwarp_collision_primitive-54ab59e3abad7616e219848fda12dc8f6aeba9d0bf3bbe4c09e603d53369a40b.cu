#pragma once
// G10@P5 G5 碰撞 GPU 内核公共数学：与 host `gjk/math.rs` 逐式对应。
// Adapted from frozen MJWarp math.py, revision 71da24d. Apache-2.0.
// 编译时用 NVRTC `--fmad=false` 可与 host f32 源码顺序逐位对照；
// GPU 判据（对上游 f32 GPU）另按 pinned PTX 的显式收缩模式编译，两模式分开记录。
#pragma once

struct v3 {
  float x, y, z;
};

__device__ __forceinline__ v3 v3_make(float x, float y, float z) {
  v3 out = {x, y, z};
  return out;
}

__device__ __forceinline__ v3 v3_add(v3 a, v3 b) {
  return v3_make(a.x + b.x, a.y + b.y, a.z + b.z);
}

__device__ __forceinline__ v3 v3_sub(v3 a, v3 b) {
  return v3_make(a.x - b.x, a.y - b.y, a.z - b.z);
}

__device__ __forceinline__ v3 v3_neg(v3 a) {
  return v3_make(-a.x, -a.y, -a.z);
}

__device__ __forceinline__ v3 v3_scale(v3 a, float s) {
  return v3_make(a.x * s, a.y * s, a.z * s);
}

__device__ __forceinline__ v3 v3_cw_mul(v3 a, v3 b) {
  return v3_make(a.x * b.x, a.y * b.y, a.z * b.z);
}

// 上游 `warp/native/builtin.h`：`sign(x) = x < 0 ? -1 : 1`（**零/-0.0 返回 +1，不返回 0**）。
// 设备侧旧实现把零映射为 0，导致 BOX 支撑在 local_dir 分量恰为 0 时取面中心而非角点
// （ellipsoid-box：GJK 首轮支撑点即分叉 → dim 4→3 → EPA 路径/witness 切向偏；见证据 §11.6）。
__device__ __forceinline__ v3 v3_sign(v3 a) {
  v3 out = {a.x < 0.0f ? -1.0f : 1.0f, a.y < 0.0f ? -1.0f : 1.0f,
            a.z < 0.0f ? -1.0f : 1.0f};
  return out;
}

__device__ __forceinline__ float v3_dot(v3 a, v3 b) {
  return a.x * b.x + a.y * b.y + a.z * b.z;
}

__device__ __forceinline__ v3 v3_cross(v3 a, v3 b) {
  return v3_make(a.y * b.z - a.z * b.y, a.z * b.x - a.x * b.z,
                 a.x * b.y - a.y * b.x);
}

__device__ __forceinline__ float v3_length(v3 a) {
  return sqrtf(v3_dot(a, a));
}

__device__ __forceinline__ v3 v3_normalize(v3 a) {
  float norm = v3_length(a);
  if (norm == 0.0f) return v3_make(0.0f, 0.0f, 0.0f);
  return v3_make(a.x / norm, a.y / norm, a.z / norm);
}

// 行优先 mat33 @ vec3（与 host `mul_mat3_vec` 一致：m[0]*v0+m[1]*v1+m[2]*v2）。
__device__ __forceinline__ v3 mul_mat3_vec(const float* m, v3 v) {
  return v3_make(m[0] * v.x + m[1] * v.y + m[2] * v.z,
                 m[3] * v.x + m[4] * v.y + m[5] * v.z,
                 m[6] * v.x + m[7] * v.y + m[8] * v.z);
}

// transpose(mat33) @ vec3（与 host `transpose_mul_mat3_vec` 一致）。
__device__ __forceinline__ v3 transpose_mul_mat3_vec(const float* m, v3 v) {
  return v3_make(m[0] * v.x + m[3] * v.y + m[6] * v.z,
                 m[1] * v.x + m[4] * v.y + m[7] * v.z,
                 m[2] * v.x + m[5] * v.y + m[8] * v.z);
}

// 上游 `relative_matrix`：`transpose(a) * b`，行优先存储（rel[row][col] = Σ_k a[k][row]*b[k][col]）。
// 累加按上游 `warp/native/mat.h::mul(mat,mat)` 的 `muladd`（fmaf）语义用显式 `__fmaf_rn` 链
// （k 升序：`sum = a0*b0; sum = fma(a1,b1,sum); sum = fma(a2,b2,sum)`）；不交给编译器按
// `--fmad` 自动收缩，避免判据位型随编译开关漂移。注意 `transpose_mul_mat3_vec`（matvec）
// 上游是普通乘加，保持原样。
// 判据是上游语义；host `sdf/descent.rs` / `hfield/driver.rs` 的旧 helper 在行优先约定下
// 返回的是转置（`bᵀa`）且用普通乘加，设备侧不复制该约定（详见 `src/physics/collision/relmat.rs`）。
__device__ __forceinline__ void relative_matrix(const float* a, const float* b,
                                                float* out) {
  for (int row = 0; row < 3; ++row) {
    for (int column = 0; column < 3; ++column) {
      float sum = a[row] * b[column];
      sum = __fmaf_rn(a[3 + row], b[3 + column], sum);
      out[row * 3 + column] = __fmaf_rn(a[6 + row], b[6 + column], sum);
    }
  }
}

// 上游 `math.make_frame`：以法线为第一行的接触坐标系（行优先）。
__device__ __forceinline__ void make_frame(v3 normal, float* frame) {
  v3 a = v3_normalize(normal);
  v3 selector = v3_make(0.0f, 1.0f, 0.0f);
  if (!(-0.5f < a.y && a.y < 0.5f)) selector = v3_make(0.0f, 0.0f, 1.0f);
  v3 b = v3_normalize(v3_sub(selector, v3_scale(a, v3_dot(a, selector))));
  if (v3_length(a) == 0.0f) b = v3_make(0.0f, 0.0f, 0.0f);
  v3 c = v3_cross(a, b);
  frame[0] = a.x; frame[1] = a.y; frame[2] = a.z;
  frame[3] = b.x; frame[4] = b.y; frame[5] = b.z;
  frame[6] = c.x; frame[7] = c.y; frame[8] = c.z;
}
// G10@P5 G5 设备侧几何视图（与 host `gjk::Geom`/`MeshView` 字段对应）。
// Adapted from frozen MJWarp collision_core.py, revision 71da24d. Apache-2.0.
#pragma once

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
// G10@P5 G5：支持点函数设备版，与 host `gjk/support.rs` 逐式对应。
// Adapted from frozen MJWarp collision_gjk.py, revision 71da24d. Apache-2.0.
#pragma once

#define G5_PLANE 0
#define G5_HFIELD 1
#define G5_SPHERE 2
#define G5_CAPSULE 3
#define G5_ELLIPSOID 4
#define G5_CYLINDER 5
#define G5_BOX 6
#define G5_MESH 7
#define G5_SDF 8
#define G5_FLEX 105
#define G5_TRIANGLE 999

#define G5_FLOAT_MIN (-1.0e30f)
#define G5_MINVAL 1.0e-15f

__device__ __forceinline__ void support_mesh(const d_geom& geom, const d_mesh& mesh,
                                             v3 local_dir, d_support_point* point) {
  unsigned vertadr = (unsigned)(geom.vertadr > 0 ? geom.vertadr : 0);
  unsigned vertnum = (unsigned)(geom.vertnum > 0 ? geom.vertnum : 0);
  float max_dist = G5_FLOAT_MIN;
  if (geom.graphadr == -1 || geom.vertnum < 10) {
    if (geom.index > -1) {
      unsigned index = (unsigned)geom.index;
      point->cached_index = geom.index;
      max_dist = v3_dot(v3_make(mesh.vert[index * 3], mesh.vert[index * 3 + 1],
                                mesh.vert[index * 3 + 2]),
                        local_dir);
      point->point = v3_make(mesh.vert[index * 3], mesh.vert[index * 3 + 1],
                             mesh.vert[index * 3 + 2]);
    }
    for (unsigned i = 0; i < vertnum; ++i) {
      unsigned base = (vertadr + i) * 3;
      v3 vert = v3_make(mesh.vert[base], mesh.vert[base + 1], mesh.vert[base + 2]);
      float dist = v3_dot(vert, local_dir);
      if (dist > max_dist) {
        max_dist = dist;
        point->point = vert;
        point->cached_index = (int)(vertadr + i);
      }
    }
    point->vertex_index = point->cached_index - geom.vertadr;
  } else {
    unsigned graphadr = (unsigned)(geom.graphadr > 0 ? geom.graphadr : 0);
    unsigned numvert = (unsigned)mesh.graph[graphadr];
    unsigned vert_edgeadr = graphadr + 2;
    unsigned vert_globalid = graphadr + 2 + numvert;
    unsigned edge_localid = graphadr + 2 + 2 * numvert;
    int prev = -1;
    unsigned imax = geom.index > -1 ? (unsigned)geom.index : 0u;
    {
      unsigned idx = (unsigned)mesh.graph[vert_globalid + imax];
      unsigned base = (vertadr + idx) * 3;
      max_dist = v3_dot(local_dir, v3_make(mesh.vert[base], mesh.vert[base + 1],
                                           mesh.vert[base + 2]));
    }
    while ((int)imax != prev) {
      prev = (int)imax;
      unsigned i = (unsigned)mesh.graph[vert_edgeadr + imax];
      int subidx = mesh.graph[edge_localid + i];
      while (subidx >= 0) {
        unsigned idx = (unsigned)mesh.graph[vert_globalid + (unsigned)subidx];
        unsigned base = (vertadr + idx) * 3;
        v3 vert = v3_make(mesh.vert[base], mesh.vert[base + 1], mesh.vert[base + 2]);
        float dist = v3_dot(local_dir, vert);
        if (dist > max_dist) {
          imax = (unsigned)subidx;
          max_dist = dist;
        }
        i += 1;
        subidx = mesh.graph[edge_localid + i];
      }
    }
    point->cached_index = (int)imax;
    point->vertex_index = mesh.graph[vert_globalid + imax];
    unsigned base = (vertadr + (unsigned)point->vertex_index) * 3;
    point->point = v3_make(mesh.vert[base], mesh.vert[base + 1], mesh.vert[base + 2]);
  }
  point->point = v3_add(mul_mat3_vec(geom.rot, point->point), geom.pos);
}

__device__ __forceinline__ d_support_point support(const d_geom& geom, int geomtype,
                                                   v3 dir) {
  d_support_point point;
  point.point = v3_make(0.0f, 0.0f, 0.0f);
  point.cached_index = -1;
  point.vertex_index = 0;

  if (geomtype == G5_SPHERE) {
    point.point = v3_add(geom.pos, v3_scale(dir, geom.size.x + 0.5f * geom.margin));
    return point;
  }

  v3 local_dir = transpose_mul_mat3_vec(geom.rot, dir);

  if (geomtype == G5_BOX) {
    v3 tmp = v3_sign(local_dir);
    v3 res = v3_cw_mul(tmp, geom.size);
    point.point = v3_add(mul_mat3_vec(geom.rot, res), geom.pos);
    int vertex_index = 0;
    if (tmp.x > 0.0f) vertex_index += 1;
    if (tmp.y > 0.0f) vertex_index += 2;
    if (tmp.z > 0.0f) vertex_index += 4;
    point.vertex_index = vertex_index;
  } else if (geomtype == G5_CAPSULE) {
    v3 res = v3_scale(local_dir, geom.size.x);
    res.z += local_dir.z < 0.0f ? -geom.size.y : geom.size.y;
    point.point = v3_add(mul_mat3_vec(geom.rot, res), geom.pos);
  } else if (geomtype == G5_ELLIPSOID) {
    v3 res = v3_normalize(v3_cw_mul(local_dir, geom.size));
    res = v3_cw_mul(res, geom.size);
    point.point = v3_add(mul_mat3_vec(geom.rot, res), geom.pos);
  } else if (geomtype == G5_CYLINDER) {
    v3 res = v3_make(0.0f, 0.0f, 0.0f);
    float d = sqrtf(local_dir.x * local_dir.x + local_dir.y * local_dir.y);
    if (d > G5_MINVAL) {
      float scl = geom.size.x / d;
      res.x = local_dir.x * scl;
      res.y = local_dir.y * scl;
    }
    bool top = local_dir.z >= 0.0f;
    res.z = top ? geom.size.y : -geom.size.y;
    point.vertex_index = top ? 0 : 1;
    point.point = v3_add(mul_mat3_vec(geom.rot, res), geom.pos);
  } else if (geomtype == G5_MESH) {
    support_mesh(geom, geom.mesh, local_dir, &point);
  } else if (geomtype == G5_HFIELD) {
    point.vertex_index = dir.z < 0.0f ? -2 : -3;
    float max_dist = G5_FLOAT_MIN;
    for (int slot = 0; slot < 6; ++slot) {
      v3 vert = v3_make(geom.polyvert[slot * 3], geom.polyvert[slot * 3 + 1],
                        geom.polyvert[slot * 3 + 2]);
      float dist = v3_dot(vert, dir);
      if (dist > max_dist) {
        max_dist = dist;
        point.point = vert;
      }
    }
  } else if (geomtype == G5_TRIANGLE) {
    v3 t1 = v3_make(geom.rot[0], geom.rot[1], geom.rot[2]);
    v3 t2 = v3_make(geom.rot[3], geom.rot[4], geom.rot[5]);
    v3 t3 = v3_make(geom.rot[6], geom.rot[7], geom.rot[8]);
    float d1 = v3_dot(t1, dir);
    float d2 = v3_dot(t2, dir);
    float d3 = v3_dot(t3, dir);
    point.point = (d1 > d2 && d1 > d3) ? t1 : (d2 > d3 ? t2 : t3);
  } else if (geomtype == G5_FLEX) {
    v3 p0 = v3_make(geom.polyvert[0], geom.polyvert[1], geom.polyvert[2]);
    v3 p1 = v3_make(geom.polyvert[3], geom.polyvert[4], geom.polyvert[5]);
    v3 p2 = v3_make(geom.polyvert[6], geom.polyvert[7], geom.polyvert[8]);
    v3 p3 = v3_make(geom.polyvert[9], geom.polyvert[10], geom.polyvert[11]);
    float d0 = v3_dot(p0, dir);
    float d1 = v3_dot(p1, dir);
    float d2 = v3_dot(p2, dir);
    float d3 = v3_dot(p3, dir);
    point.point = (d0 > d1 && d0 > d2 && d0 > d3)
                      ? p0
                      : (d1 > d2 && d1 > d3 ? p1 : (d2 > d3 ? p2 : p3));
  }

  if (geom.margin > 0.0f) {
    point.point = v3_add(point.point, v3_scale(dir, 0.5f * geom.margin));
  }
  return point;
}

__device__ __forceinline__ void support_pair(const d_geom& geom1, const d_geom& geom2,
                                             int geomtype1, int geomtype2, v3 dir,
                                             d_support_point* p1,
                                             d_support_point* p2) {
  *p1 = support(geom1, geomtype1, dir);
  *p2 = support(geom2, geomtype2, v3_neg(dir));
}

__device__ __forceinline__ bool discrete_geoms(int type1, int type2) {
  bool d1 = type1 == G5_MESH || type1 == G5_BOX || type1 == G5_HFIELD;
  bool d2 = type2 == G5_MESH || type2 == G5_BOX || type2 == G5_HFIELD;
  return d1 && d2;
}
// G10@P5 G5：距离子算法设备版，与 host `gjk/simplex.rs` 逐式对应。
// Adapted from frozen MJWarp collision_gjk.py, revision 71da24d. Apache-2.0.
#pragma once

#define G5_FLOAT_MAX 1.0e30f
#define G5_MINVAL 1.0e-15f

__device__ __forceinline__ float g5_det3(v3 v1, v3 v2, v3 v3c) {
  return v3_dot(v1, v3_cross(v2, v3c));
}

__device__ __forceinline__ int g5_same_sign(float a, float b) {
  if (a > 0.0f && b > 0.0f) return 1;
  if (a < 0.0f && b < 0.0f) return -1;
  return 0;
}

__device__ __forceinline__ v3 g5_project_origin_line(v3 v1, v3 v2) {
  v3 diff = v3_sub(v2, v1);
  float scl = -(v3_dot(v2, diff) / v3_dot(diff, diff));
  return v3_make(v2.x + scl * diff.x, v2.y + scl * diff.y, v2.z + scl * diff.z);
}

__device__ __forceinline__ void g5_scale_plane(v3 n, float nv, float nn, v3* out) {
  float scl = nv / nn;
  *out = v3_make(scl * n.x, scl * n.y, scl * n.z);
}

// 返回 true 表示退化（与 host `project_origin_plane` 的 bool 一致）。
__device__ __forceinline__ bool g5_project_origin_plane(v3 v1, v3 v2, v3 v3c,
                                                       v3* out) {
  v3 zero = v3_make(0.0f, 0.0f, 0.0f);
  v3 diff21 = v3_sub(v2, v1);
  v3 diff31 = v3_sub(v3c, v1);
  v3 diff32 = v3_sub(v3c, v2);

  v3 n = v3_cross(diff32, diff21);
  float nv = v3_dot(n, v2);
  float nn = v3_dot(n, n);
  if (nn == 0.0f) {
    *out = zero;
    return true;
  }
  if (nv != 0.0f && nn > G5_MINVAL) {
    g5_scale_plane(n, nv, nn, out);
    return false;
  }

  n = v3_cross(diff21, diff31);
  nv = v3_dot(n, v1);
  nn = v3_dot(n, n);
  if (nn == 0.0f) {
    *out = zero;
    return true;
  }
  if (nv != 0.0f && nn > G5_MINVAL) {
    g5_scale_plane(n, nv, nn, out);
    return false;
  }

  n = v3_cross(diff31, diff32);
  nv = v3_dot(n, v3c);
  nn = v3_dot(n, n);
  g5_scale_plane(n, nv, nn, out);
  return false;
}

__device__ __forceinline__ void g5_s1d(v3 s1, v3 s2, float* out) {
  v3 p_o = g5_project_origin_line(s1, s2);
  float mu_max = s1.x - s2.x;
  int index = 0;
  float mu = s1.y - s2.y;
  if (fabsf(mu) >= fabsf(mu_max)) {
    mu_max = mu;
    index = 1;
  }
  mu = s1.z - s2.z;
  if (fabsf(mu) >= fabsf(mu_max)) {
    mu_max = mu;
    index = 2;
  }
  const float* po = reinterpret_cast<const float*>(&p_o.x);
  const float* a = reinterpret_cast<const float*>(&s1.x);
  const float* b = reinterpret_cast<const float*>(&s2.x);
  float c1 = po[index] - b[index];
  float c2 = a[index] - po[index];
  if (g5_same_sign(mu_max, c1) != 0 && g5_same_sign(mu_max, c2) != 0) {
    out[0] = c1 / mu_max;
    out[1] = c2 / mu_max;
    return;
  }
  out[0] = 0.0f;
  out[1] = 1.0f;
}

__device__ __forceinline__ void g5_s2d(v3 s1, v3 s2, v3 s3, float* out) {
  v3 p_o;
  bool degenerate = g5_project_origin_plane(s1, s2, s3, &p_o);
  if (degenerate) {
    float values[2];
    g5_s1d(s1, s2, values);
    out[0] = values[0];
    out[1] = values[1];
    out[2] = 0.0f;
    return;
  }
  float m_14 = s2.y * s3.z - s2.z * s3.y - s1.y * s3.z + s1.z * s3.y + s1.y * s2.z -
               s1.z * s2.y;
  float m_24 = s2.x * s3.z - s2.z * s3.x - s1.x * s3.z + s1.z * s3.x + s1.x * s2.z -
               s1.z * s2.x;
  float m_34 = s2.x * s3.y - s2.y * s3.x - s1.x * s3.y + s1.y * s3.x + s1.x * s2.y -
               s1.y * s2.x;
  float mu1 = fabsf(m_14);
  float mu2 = fabsf(m_24);
  float mu3 = fabsf(m_34);
  float m_max;
  int x;
  int y;
  if (mu1 >= mu2 && mu1 >= mu3) {
    m_max = m_14;
    x = 1;
    y = 2;
  } else if (mu2 >= mu3) {
    m_max = m_24;
    x = 0;
    y = 2;
  } else {
    m_max = m_34;
    x = 0;
    y = 1;
  }
  const float* s1v = reinterpret_cast<const float*>(&s1.x);
  const float* s2v = reinterpret_cast<const float*>(&s2.x);
  const float* s3v = reinterpret_cast<const float*>(&s3.x);
  const float* pov = reinterpret_cast<const float*>(&p_o.x);
  float s1_2d[2] = {s1v[x], s1v[y]};
  float s2_2d[2] = {s2v[x], s2v[y]};
  float s3_2d[2] = {s3v[x], s3v[y]};
  float p_o_2d[2] = {pov[x], pov[y]};
  float c31 = p_o_2d[0] * s2_2d[1] + p_o_2d[1] * s3_2d[0] + s2_2d[0] * s3_2d[1] -
              p_o_2d[0] * s3_2d[1] - p_o_2d[1] * s2_2d[0] - s3_2d[0] * s2_2d[1];
  float c32 = p_o_2d[0] * s3_2d[1] + p_o_2d[1] * s1_2d[0] + s3_2d[0] * s1_2d[1] -
              p_o_2d[0] * s1_2d[1] - p_o_2d[1] * s3_2d[0] - s1_2d[0] * s3_2d[1];
  float c33 = p_o_2d[0] * s1_2d[1] + p_o_2d[1] * s2_2d[0] + s1_2d[0] * s2_2d[1] -
              p_o_2d[0] * s2_2d[1] - p_o_2d[1] * s1_2d[0] - s2_2d[0] * s1_2d[1];
  int comp1 = g5_same_sign(m_max, c31);
  int comp2 = g5_same_sign(m_max, c32);
  int comp3 = g5_same_sign(m_max, c33);
  if (comp1 != 0 && comp2 != 0 && comp3 != 0) {
    out[0] = c31 / m_max;
    out[1] = c32 / m_max;
    out[2] = c33 / m_max;
    return;
  }
  float dmin = G5_FLOAT_MAX;
  out[0] = 0.0f;
  out[1] = 0.0f;
  out[2] = 0.0f;
  if (comp1 == 0) {
    float sub[2];
    g5_s1d(s2, s3, sub);
    v3 x = v3_make(sub[0] * s2.x + sub[1] * s3.x, sub[0] * s2.y + sub[1] * s3.y,
                   sub[0] * s2.z + sub[1] * s3.z);
    float d = v3_dot(x, x);
    out[0] = 0.0f;
    out[1] = sub[0];
    out[2] = sub[1];
    dmin = d;
  }
  if (comp2 == 0) {
    float sub[2];
    g5_s1d(s1, s3, sub);
    v3 x = v3_make(sub[0] * s1.x + sub[1] * s3.x, sub[0] * s1.y + sub[1] * s3.y,
                   sub[0] * s1.z + sub[1] * s3.z);
    float d = v3_dot(x, x);
    if (d < dmin) {
      out[0] = sub[0];
      out[1] = 0.0f;
      out[2] = sub[1];
      dmin = d;
    }
  }
  if (comp3 == 0) {
    float sub[2];
    g5_s1d(s1, s2, sub);
    v3 x = v3_make(sub[0] * s1.x + sub[1] * s2.x, sub[0] * s1.y + sub[1] * s2.y,
                   sub[0] * s1.z + sub[1] * s2.z);
    float d = v3_dot(x, x);
    if (d < dmin) {
      out[0] = sub[0];
      out[1] = sub[1];
      out[2] = 0.0f;
    }
  }
}

__device__ __forceinline__ void g5_s3d(v3 s1, v3 s2, v3 s3, v3 s4, float* out) {
  float c41 = -g5_det3(s2, s3, s4);
  float c42 = g5_det3(s1, s3, s4);
  float c43 = -g5_det3(s1, s2, s4);
  float c44 = g5_det3(s1, s2, s3);
  float m_det = c41 + c42 + c43 + c44;
  int comp1 = g5_same_sign(m_det, c41);
  int comp2 = g5_same_sign(m_det, c42);
  int comp3 = g5_same_sign(m_det, c43);
  int comp4 = g5_same_sign(m_det, c44);
  if (comp1 != 0 && comp2 != 0 && comp3 != 0 && comp4 != 0) {
    out[0] = c41 / m_det;
    out[1] = c42 / m_det;
    out[2] = c43 / m_det;
    out[3] = c44 / m_det;
    return;
  }
  float lmbda[4] = {0.0f, 0.0f, 0.0f, 0.0f};
  float dmin = G5_FLOAT_MAX;
  if (comp1 == 0) {
    float sub[3];
    g5_s2d(s2, s3, s4, sub);
    v3 x = v3_make(sub[0] * s2.x + sub[1] * s3.x + sub[2] * s4.x,
                   sub[0] * s2.y + sub[1] * s3.y + sub[2] * s4.y,
                   sub[0] * s2.z + sub[1] * s3.z + sub[2] * s4.z);
    float d = v3_dot(x, x);
    lmbda[0] = 0.0f;
    lmbda[1] = sub[0];
    lmbda[2] = sub[1];
    lmbda[3] = sub[2];
    dmin = d;
  }
  if (comp2 == 0) {
    float sub[3];
    g5_s2d(s1, s3, s4, sub);
    v3 x = v3_make(sub[0] * s1.x + sub[1] * s3.x + sub[2] * s4.x,
                   sub[0] * s1.y + sub[1] * s3.y + sub[2] * s4.y,
                   sub[0] * s1.z + sub[1] * s3.z + sub[2] * s4.z);
    float d = v3_dot(x, x);
    if (d < dmin) {
      lmbda[0] = sub[0];
      lmbda[1] = 0.0f;
      lmbda[2] = sub[1];
      lmbda[3] = sub[2];
      dmin = d;
    }
  }
  if (comp3 == 0) {
    float sub[3];
    g5_s2d(s1, s2, s4, sub);
    v3 x = v3_make(sub[0] * s1.x + sub[1] * s2.x + sub[2] * s4.x,
                   sub[0] * s1.y + sub[1] * s2.y + sub[2] * s4.y,
                   sub[0] * s1.z + sub[1] * s2.z + sub[2] * s4.z);
    float d = v3_dot(x, x);
    if (d < dmin) {
      lmbda[0] = sub[0];
      lmbda[1] = sub[1];
      lmbda[2] = 0.0f;
      lmbda[3] = sub[2];
      dmin = d;
    }
  }
  if (comp4 == 0) {
    float sub[3];
    g5_s2d(s1, s2, s3, sub);
    v3 x = v3_make(sub[0] * s1.x + sub[1] * s2.x + sub[2] * s3.x,
                   sub[0] * s1.y + sub[1] * s2.y + sub[2] * s3.y,
                   sub[0] * s1.z + sub[1] * s2.z + sub[2] * s3.z);
    float d = v3_dot(x, x);
    if (d < dmin) {
      lmbda[0] = sub[0];
      lmbda[1] = sub[1];
      lmbda[2] = sub[2];
      lmbda[3] = 0.0f;
    }
  }
  out[0] = lmbda[0];
  out[1] = lmbda[1];
  out[2] = lmbda[2];
  out[3] = lmbda[3];
}

__device__ __forceinline__ v3 g5_linear_combine(int n, const float* scl,
                                                const v3* matrix) {
  if (n == 1) {
    return v3_make(scl[0] * matrix[0].x, scl[0] * matrix[0].y, scl[0] * matrix[0].z);
  }
  if (n == 2) {
    return v3_make(scl[0] * matrix[0].x + scl[1] * matrix[1].x,
                   scl[0] * matrix[0].y + scl[1] * matrix[1].y,
                   scl[0] * matrix[0].z + scl[1] * matrix[1].z);
  }
  if (n == 3) {
    return v3_make(
        scl[0] * matrix[0].x + scl[1] * matrix[1].x + scl[2] * matrix[2].x,
        scl[0] * matrix[0].y + scl[1] * matrix[1].y + scl[2] * matrix[2].y,
        scl[0] * matrix[0].z + scl[1] * matrix[1].z + scl[2] * matrix[2].z);
  }
  return v3_make(scl[0] * matrix[0].x + scl[1] * matrix[1].x + scl[2] * matrix[2].x +
                     scl[3] * matrix[3].x,
                 scl[0] * matrix[0].y + scl[1] * matrix[1].y + scl[2] * matrix[2].y +
                     scl[3] * matrix[3].y,
                 scl[0] * matrix[0].z + scl[1] * matrix[1].z + scl[2] * matrix[2].z +
                     scl[3] * matrix[3].z);
}

__device__ __forceinline__ void g5_subdistance(int n, const v3* simplex,
                                               float* out) {
  out[0] = 1.0f;
  out[1] = 0.0f;
  out[2] = 0.0f;
  out[3] = 0.0f;
  if (n == 4) {
    g5_s3d(simplex[0], simplex[1], simplex[2], simplex[3], out);
  } else if (n == 3) {
    float values[3];
    g5_s2d(simplex[0], simplex[1], simplex[2], values);
    out[0] = values[0];
    out[1] = values[1];
    out[2] = values[2];
  } else if (n == 2) {
    float values[2];
    g5_s1d(simplex[0], simplex[1], values);
    out[0] = values[0];
    out[1] = values[1];
  }
}

__device__ __forceinline__ void g5_tri_affine_coord(v3 v1, v3 v2, v3 v3c, v3 p,
                                                    float* out) {
  float m_14 = v2.y * v3c.z - v2.z * v3c.y - v1.y * v3c.z + v1.z * v3c.y + v1.y * v2.z -
               v1.z * v2.y;
  float m_24 = v2.x * v3c.z - v2.z * v3c.x - v1.x * v3c.z + v1.z * v3c.x + v1.x * v2.z -
               v1.z * v2.x;
  float m_34 = v2.x * v3c.y - v2.y * v3c.x - v1.x * v3c.y + v1.y * v3c.x + v1.x * v2.y -
               v1.y * v2.x;
  float mu1 = fabsf(m_14);
  float mu2 = fabsf(m_24);
  float mu3 = fabsf(m_34);
  float m_max;
  int x;
  int y;
  if (mu1 >= mu2 && mu1 >= mu3) {
    m_max = m_14;
    x = 1;
    y = 2;
  } else if (mu2 >= mu3) {
    m_max = m_24;
    x = 0;
    y = 2;
  } else {
    m_max = m_34;
    x = 0;
    y = 1;
  }
  const float* pv = reinterpret_cast<const float*>(&p.x);
  const float* v1v = reinterpret_cast<const float*>(&v1.x);
  const float* v2v = reinterpret_cast<const float*>(&v2.x);
  const float* v3v = reinterpret_cast<const float*>(&v3c.x);
  float c31 = pv[x] * v2v[y] + pv[y] * v3v[x] + v2v[x] * v3v[y] - pv[x] * v3v[y] -
              pv[y] * v2v[x] - v3v[x] * v2v[y];
  float c32 = pv[x] * v3v[y] + pv[y] * v1v[x] + v3v[x] * v1v[y] - pv[x] * v1v[y] -
              pv[y] * v3v[x] - v1v[x] * v3v[y];
  float c33 = pv[x] * v1v[y] + pv[y] * v2v[x] + v1v[x] * v2v[y] - pv[x] * v2v[y] -
              pv[y] * v1v[x] - v2v[x] * v1v[y];
  out[0] = c31 / m_max;
  out[1] = c32 / m_max;
  out[2] = c33 / m_max;
}

__device__ __forceinline__ bool g5_tri_point_intersect(v3 v1, v3 v2, v3 v3c, v3 p) {
  float coordinates[3];
  g5_tri_affine_coord(v1, v2, v3c, p, coordinates);
  float l1 = coordinates[0];
  float l2 = coordinates[1];
  float l3 = coordinates[2];
  if (l1 < 0.0f || l2 < 0.0f || l3 < 0.0f) return false;
  v3 pr = v3_make(v1.x * l1 + v2.x * l2 + v3c.x * l3,
                  v1.y * l1 + v2.y * l2 + v3c.y * l3,
                  v1.z * l1 + v2.z * l2 + v3c.z * l3);
  return v3_length(v3_sub(pr, p)) < G5_MINVAL;
}

__device__ __forceinline__ bool g5_same_side(v3 p0, v3 p1, v3 p2, v3 p3) {
  v3 n = v3_cross(v3_sub(p1, p0), v3_sub(p2, p0));
  float dot1 = v3_dot(n, v3_sub(p3, p0));
  float dot2 = v3_dot(n, v3_neg(p0));
  return (dot1 > 0.0f && dot2 > 0.0f) || (dot1 < 0.0f && dot2 < 0.0f);
}

__device__ __forceinline__ bool g5_test_tetra(v3 p0, v3 p1, v3 p2, v3 p3) {
  return g5_same_side(p0, p1, p2, p3) && g5_same_side(p1, p2, p3, p0) &&
         g5_same_side(p2, p3, p0, p1) && g5_same_side(p3, p0, p1, p2);
}
// G10@P5 G5：GJK 主循环设备版，与 host `gjk/core.rs` 逐式对应。
// Adapted from frozen MJWarp collision_gjk.py, revision 71da24d. Apache-2.0.
#pragma once

#define G5_FLOAT_MAX 1.0e30f
#define G5_MINVAL2 1.0e-30f

struct d_gjk_result {
  bool separated;
  float dist;
  v3 x1;
  v3 x2;
  int dim;
  v3 simplex[4];
  v3 simplex1[4];
  v3 simplex2[4];
  int simplex_index1[4];
  int simplex_index2[4];
  int index1;
  int index2;
};

__device__ __forceinline__ d_gjk_result g5_gjk_result_zero() {
  d_gjk_result result;
  result.separated = false;
  result.dist = 0.0f;
  result.x1 = v3_make(0.0f, 0.0f, 0.0f);
  result.x2 = v3_make(0.0f, 0.0f, 0.0f);
  result.dim = 0;
  for (int i = 0; i < 4; ++i) {
    result.simplex[i] = v3_make(0.0f, 0.0f, 0.0f);
    result.simplex1[i] = v3_make(0.0f, 0.0f, 0.0f);
    result.simplex2[i] = v3_make(0.0f, 0.0f, 0.0f);
    result.simplex_index1[i] = 0;
    result.simplex_index2[i] = 0;
  }
  result.index1 = 0;
  result.index2 = 0;
  return result;
}

// 上游 `_gjk_support`：按当前 `x_k` 取两侧支撑点，离散几何做方向调优。
__device__ __forceinline__ void g5_gjk_support(const d_geom& geom1, const d_geom& geom2,
                                               int geomtype1, int geomtype2, v3 x_k,
                                               float x_norm, const v3* simplex, int n,
                                               bool is_discrete, d_support_point* sp1,
                                               d_support_point* sp2) {
  v3 dir_neg = v3_make(x_k.x / x_norm, x_k.y / x_norm, x_k.z / x_norm);
  if (is_discrete && x_norm < 1e-4f) {
    if (n == 2) {
      v3 edge = v3_sub(simplex[1], simplex[0]);
      float edge_norm2 = v3_dot(edge, edge);
      if (edge_norm2 > G5_MINVAL2) {
        float proj = v3_dot(dir_neg, edge) / edge_norm2;
        dir_neg = v3_sub(dir_neg, v3_scale(edge, proj));
        float dir_norm = v3_length(dir_neg);
        if (dir_norm > G5_MINVAL) {
          dir_neg = v3_make(dir_neg.x / dir_norm, dir_neg.y / dir_norm, dir_neg.z / dir_norm);
        }
      }
    } else if (n == 3) {
      v3 e1 = v3_sub(simplex[1], simplex[0]);
      v3 e2 = v3_sub(simplex[2], simplex[0]);
      v3 normal = v3_cross(e1, e2);
      float normal_norm = v3_length(normal);
      if (normal_norm > G5_MINVAL) {
        float sign = v3_dot(dir_neg, normal) < 0.0f ? -1.0f : 1.0f;
        dir_neg = v3_make(sign * normal.x / normal_norm, sign * normal.y / normal_norm,
                          sign * normal.z / normal_norm);
      }
    }
  }
  *sp1 = support(geom1, geomtype1, v3_neg(dir_neg));
  *sp2 = support(geom2, geomtype2, dir_neg);
}

// 上游 `gjk()`：两凸几何距离；返回结构与 host `GjkResult` 对应。
__device__ __forceinline__ d_gjk_result g5_gjk(float tolerance, int gjk_iterations,
                                               d_geom geom1, d_geom geom2, v3 x1_0,
                                               v3 x2_0, int geomtype1, int geomtype2,
                                               float cutoff, bool is_discrete) {
  d_gjk_result result = g5_gjk_result_zero();
  v3 simplex[4];
  v3 simplex1[4];
  v3 simplex2[4];
  int simplex_index1[4];
  int simplex_index2[4];
  for (int i = 0; i < 4; ++i) {
    simplex[i] = v3_make(0.0f, 0.0f, 0.0f);
    simplex1[i] = v3_make(0.0f, 0.0f, 0.0f);
    simplex2[i] = v3_make(0.0f, 0.0f, 0.0f);
    simplex_index1[i] = 0;
    simplex_index2[i] = 0;
  }
  int n = 0;
  float lmbda[4] = {1.0f, 0.0f, 0.0f, 0.0f};

  float epsilon = is_discrete ? 0.0f : 0.5f * tolerance * tolerance;
  float min_norm = is_discrete ? G5_MINVAL : tolerance;

  v3 x_k = v3_sub(x1_0, x2_0);
  float xnorm = sqrtf(v3_dot(x_k, x_k));
  float xnorm_prev = 0.0f;

  for (int step = 0; step < gjk_iterations; ++step) {
    if (xnorm < min_norm || fabsf(xnorm_prev - xnorm) < G5_MINVAL) break;

    d_support_point sp1;
    d_support_point sp2;
    g5_gjk_support(geom1, geom2, geomtype1, geomtype2, x_k, xnorm, simplex, n,
                   is_discrete, &sp1, &sp2);
    simplex1[n] = sp1.point;
    geom1.index = sp1.cached_index;
    simplex_index1[n] = sp1.vertex_index;
    simplex2[n] = sp2.point;
    geom2.index = sp2.cached_index;
    simplex_index2[n] = sp2.vertex_index;
    simplex[n] = v3_sub(simplex1[n], simplex2[n]);

    // Frank-Wolfe 对偶间隙停止判据。
    if (v3_dot(x_k, v3_sub(x_k, simplex[n])) < epsilon) break;

    // 距离下界：lower > 0 表示分离。
    float lower = v3_dot(x_k, simplex[n]);
    if (cutoff == 0.0f) {
      if (lower > 0.0f) {
        result.separated = true;
        result.dim = 0;
        result.dist = G5_FLOAT_MAX;
        result.index1 = geom1.index;
        result.index2 = geom2.index;
        return result;
      }
    } else if (cutoff < G5_FLOAT_MAX && lower > 0.0f && lower >= cutoff * xnorm) {
      result.separated = true;
      result.dim = 0;
      result.dist = G5_FLOAT_MAX;
      result.index1 = geom1.index;
      result.index2 = geom2.index;
      return result;
    }

    // 距离子算法：simplex 中离原点最近点的重心坐标。
    g5_subdistance(n + 1, simplex, lmbda);

    // 剔除不再需要的顶点。
    n = 0;
    for (int i = 0; i < 4; ++i) {
      if (lmbda[i] == 0.0f) continue;
      simplex[n] = simplex[i];
      simplex1[n] = simplex1[i];
      simplex2[n] = simplex2[i];
      simplex_index1[n] = simplex_index1[i];
      simplex_index2[n] = simplex_index2[i];
      lmbda[n] = lmbda[i];
      n += 1;
    }
    if (n < 1) break;

    x_k = g5_linear_combine(n, lmbda, simplex);
    xnorm_prev = xnorm;
    xnorm = sqrtf(v3_dot(x_k, x_k));
    if (n == 4) break;
  }

  result.separated = false;
  if (n == 0) {
    result.x1 = x1_0;
    result.x2 = x2_0;
  } else {
    result.x1 = g5_linear_combine(n, lmbda, simplex1);
    result.x2 = g5_linear_combine(n, lmbda, simplex2);
  }
  if (xnorm > 0.0f) {
    v3 dir = v3_make(x_k.x / xnorm, x_k.y / xnorm, x_k.z / xnorm);
    d_support_point sp1 = support(geom1, geomtype1, v3_neg(dir));
    d_support_point sp2 = support(geom2, geomtype2, dir);
    result.separated = v3_dot(x_k, v3_sub(sp1.point, sp2.point)) > 0.0f;
  }
  result.dist = (n == 4 && !result.separated) ? 0.0f : xnorm;
  result.dim = n;
  for (int i = 0; i < 4; ++i) {
    result.simplex[i] = simplex[i];
    result.simplex1[i] = simplex1[i];
    result.simplex2[i] = simplex2[i];
    result.simplex_index1[i] = simplex_index1[i];
    result.simplex_index2[i] = simplex_index2[i];
  }
  result.index1 = geom1.index;
  result.index2 = geom2.index;
  return result;
}

// 上游 `_gjk_support` 的离散几何判定（供 `phase` 复用）。
__device__ __forceinline__ bool g5_is_discrete(int geomtype1, int geomtype2, float margin1,
                                               float margin2) {
  return discrete_geoms(geomtype1, geomtype2) && margin1 == 0.0f && margin2 == 0.0f;
}
// G10@P5 G5：EPA polytope 构造与 witness 恢复设备版，与 host `gjk/polytope.rs` 逐式对应。
// Adapted from frozen MJWarp collision_gjk.py, revision 71da24d. Apache-2.0.
#pragma once

#define G5_FACE_DELETED_BIT 0x80000000u
#define G5_FACE_INVALID_BIT 0x40000000u
#define G5_FACE_INVALID_OR_DELETED_MASK 0xC0000000u

// 缓冲区由调用方（宿主 launcher）按 epa_iterations 分配，容量在 max_face 中给出。
struct d_polytope {
  int nvert;
  int nface;
  int nhorizon;
  int status;
  v3* vert;        // 2*i = geom1 支撑点，2*i+1 = geom2 支撑点
  int* vert_index;
  unsigned* face;
  v3* face_pr;
  float* face_norm2;
  int* horizon;
  v3 center;
  int max_face;
  int max_horizon;
};

__device__ __forceinline__ void g5_get_face_verts(unsigned face, int* out) {
  out[0] = (int)(face & 0x3FFu);
  out[1] = (int)((face >> 10) & 0x3FFu);
  out[2] = (int)((face >> 20) & 0x3FFu);
}

__device__ __forceinline__ unsigned g5_delete_face(unsigned face) {
  return face | G5_FACE_DELETED_BIT;
}

__device__ __forceinline__ bool g5_is_face_deleted(unsigned face) {
  return (face & G5_FACE_DELETED_BIT) != 0u;
}

__device__ __forceinline__ unsigned g5_invalidate_face(unsigned face) {
  return face | G5_FACE_INVALID_BIT;
}

__device__ __forceinline__ bool g5_is_invalid_face(unsigned face) {
  return (face & G5_FACE_INVALID_OR_DELETED_MASK) != 0u;
}

// 上游 `_attach_face`：返回原点投影距离平方（0 = 失败/退化）。
__device__ __forceinline__ float g5_attach_face(d_polytope* pt, int idx, int idx1,
                                                int idx2, int idx3) {
  if (pt->nface == pt->max_face) return 0.0f;
  v3 p1 = v3_sub(pt->vert[2 * idx1], pt->vert[2 * idx1 + 1]);
  v3 p2 = v3_sub(pt->vert[2 * idx2], pt->vert[2 * idx2 + 1]);
  v3 p3 = v3_sub(pt->vert[2 * idx3], pt->vert[2 * idx3 + 1]);
  v3 r;
  bool degenerate = g5_project_origin_plane(p3, p2, p1, &r);
  if (degenerate) return 0.0f;
  if (v3_dot(r, v3_sub(p1, pt->center)) < 0.0f) r = v3_neg(r);
  pt->face[idx] = (unsigned)idx1 + ((unsigned)idx2 << 10) + ((unsigned)idx3 << 20);
  pt->face_pr[idx] = r;
  // 对齐 pinned ccd PTX 的 FMA 收缩：dot(r,r) 为「首项乘积 + 逐项 fma」链。
  float head = r.x * r.x;
  float tail = fmaf(r.y, r.y, head);
  pt->face_norm2[idx] = fmaf(r.z, r.z, tail);
  return pt->face_norm2[idx];
}

// 上游 `_epa_support`：写入 polytope 第 idx 对顶点，返回两侧缓存索引。
__device__ __forceinline__ void g5_epa_support(d_polytope* pt, int idx,
                                               const d_geom& geom1, const d_geom& geom2,
                                               int geomtype1, int geomtype2, v3 dir,
                                               int* index1, int* index2) {
  d_support_point sp = support(geom1, geomtype1, dir);
  pt->vert[2 * idx] = sp.point;
  pt->vert_index[2 * idx] = sp.vertex_index;
  *index1 = sp.cached_index;
  sp = support(geom2, geomtype2, v3_neg(dir));
  pt->vert[2 * idx + 1] = sp.point;
  pt->vert_index[2 * idx + 1] = sp.vertex_index;
  *index2 = sp.cached_index;
}

// 上游 `_rotmat`：绕轴旋转 120° 的矩阵（行优先）。
__device__ __forceinline__ void g5_rotmat(v3 axis, float* out) {
  float n = v3_length(axis);
  float u1 = axis.x / n;
  float u2 = axis.y / n;
  float u3 = axis.z / n;
  float sin_v = 0.86602540378f;
  float cos_v = -0.5f;
  float one_minus = 1.0f - cos_v;
  out[0] = cos_v + u1 * u1 * one_minus;
  out[1] = u1 * u2 * one_minus - u3 * sin_v;
  out[2] = u1 * u3 * one_minus + u2 * sin_v;
  out[3] = u2 * u1 * one_minus + u3 * sin_v;
  out[4] = cos_v + u2 * u2 * one_minus;
  out[5] = u2 * u3 * one_minus - u1 * sin_v;
  out[6] = u1 * u3 * one_minus - u2 * sin_v;
  out[7] = u2 * u3 * one_minus + u1 * sin_v;
  out[8] = cos_v + u3 * u3 * one_minus;
}

// 上游 `_ray_triangle`：返回 1/-1/0。
__device__ __forceinline__ int g5_ray_triangle(v3 p1, v3 p2, v3 p3, v3 p4, v3 p5) {
  float vol1 = g5_det3(v3_sub(p3, p1), v3_sub(p4, p1), v3_sub(p2, p1));
  float vol2 = g5_det3(v3_sub(p4, p1), v3_sub(p5, p1), v3_sub(p2, p1));
  float vol3 = g5_det3(v3_sub(p5, p1), v3_sub(p3, p1), v3_sub(p2, p1));
  if (vol1 >= 0.0f && vol2 >= 0.0f && vol3 >= 0.0f) return 1;
  if (vol1 <= 0.0f && vol2 <= 0.0f && vol3 <= 0.0f) return -1;
  return 0;
}

// 上游 `_replace_simplex3`：用 polytope 的 3 顶点重置 GJK 结果。
__device__ __forceinline__ d_gjk_result g5_replace_simplex3(const d_polytope* pt, int idx1,
                                                            int idx2, int idx3) {
  d_gjk_result result = g5_gjk_result_zero();
  result.simplex1[0] = pt->vert[2 * idx1];
  result.simplex1[1] = pt->vert[2 * idx2];
  result.simplex1[2] = pt->vert[2 * idx3];
  result.simplex2[0] = pt->vert[2 * idx1 + 1];
  result.simplex2[1] = pt->vert[2 * idx2 + 1];
  result.simplex2[2] = pt->vert[2 * idx3 + 1];
  result.simplex[0] = v3_sub(result.simplex1[0], result.simplex2[0]);
  result.simplex[1] = v3_sub(result.simplex1[1], result.simplex2[1]);
  result.simplex[2] = v3_sub(result.simplex1[2], result.simplex2[2]);
  result.simplex_index1[0] = pt->vert_index[2 * idx1];
  result.simplex_index1[1] = pt->vert_index[2 * idx2];
  result.simplex_index1[2] = pt->vert_index[2 * idx3];
  result.simplex_index2[0] = pt->vert_index[2 * idx1 + 1];
  result.simplex_index2[1] = pt->vert_index[2 * idx2 + 1];
  result.simplex_index2[2] = pt->vert_index[2 * idx3 + 1];
  return result;
}

// 上游 `_epa_witness`：从选中面恢复 witness 点与穿透距离。
__device__ __forceinline__ void g5_epa_witness(const d_polytope* pt, const d_geom& geom1,
                                               const d_geom& geom2, int geomtype1,
                                               int geomtype2, int face_idx, v3* out_x1,
                                               v3* out_x2, float* out_dist) {
  int face[3];
  g5_get_face_verts(pt->face[face_idx], face);
  v3 v1 = v3_sub(pt->vert[2 * face[0]], pt->vert[2 * face[0] + 1]);
  v3 v2 = v3_sub(pt->vert[2 * face[1]], pt->vert[2 * face[1] + 1]);
  v3 v3v = v3_sub(pt->vert[2 * face[2]], pt->vert[2 * face[2] + 1]);
  float coordinates[3];
  g5_tri_affine_coord(v1, v2, v3v, pt->face_pr[face_idx], coordinates);
  float l1 = coordinates[0];
  float l2 = coordinates[1];
  float l3 = coordinates[2];

  v3 w1 = pt->vert[2 * face[0] + 1];
  v3 w2 = pt->vert[2 * face[1] + 1];
  v3 w3 = pt->vert[2 * face[2] + 1];
  v3 x2 = v3_make(w1.x * l1 + w2.x * l2 + w3.x * l3,
                  w1.y * l1 + w2.y * l2 + w3.y * l3,
                  w1.z * l1 + w2.z * l2 + w3.z * l3);

  int i1 = pt->vert_index[2 * face[0]];
  int i2 = pt->vert_index[2 * face[1]];
  int i3 = pt->vert_index[2 * face[2]];
  if (geomtype1 == G5_HFIELD && (i1 != i2 || i1 != i3)) {
    v3 n = v3_make(0.0f, 0.0f, 1.0f);
    v3 a = v3_make(geom1.polyvert[9], geom1.polyvert[10], geom1.polyvert[11]);
    v3 b = v3_make(geom1.polyvert[12], geom1.polyvert[13], geom1.polyvert[14]);
    v3 c = v3_make(geom1.polyvert[15], geom1.polyvert[16], geom1.polyvert[17]);
    v3 shifted_x2;
    if (geomtype2 == G5_CAPSULE || geomtype2 == G5_SPHERE) {
      float radius = geom2.size.x;
      float margin = geom2.margin;
      d_geom geom2_shifted = geom2;
      geom2_shifted.margin = 0.0f;
      geom2_shifted.size = v3_make(0.0f, geom2.size.y, geom2.size.z);
      d_support_point sp = support(geom2_shifted, geomtype2, x2);
      float offset = 0.5f * margin + radius;
      shifted_x2 = v3_make(sp.point.x - offset * n.x, sp.point.y - offset * n.y,
                           sp.point.z - offset * n.z);
    } else {
      shifted_x2 = support(geom2, geomtype2, v3_normalize(x2)).point;
    }
    float coordinates2[3];
    g5_tri_affine_coord(a, b, c, shifted_x2, coordinates2);
    v3 x1;
    if (coordinates2[0] > 0.0f && coordinates2[1] > 0.0f && coordinates2[2] > 0.0f) {
      x1 = v3_make(coordinates2[0] * a.x + coordinates2[1] * b.x + coordinates2[2] * c.x,
                   coordinates2[0] * a.y + coordinates2[1] * b.y + coordinates2[2] * c.y,
                   coordinates2[0] * a.z + coordinates2[1] * b.z + coordinates2[2] * c.z);
    } else {
      v3 p = c;
      if (coordinates2[1] > 0.0f) p = b;
      if (coordinates2[0] > 0.0f) p = a;
      float d = v3_dot(v3_sub(shifted_x2, p), n);
      x1 = v3_make(shifted_x2.x - d * n.x, shifted_x2.y - d * n.y,
                   shifted_x2.z - d * n.z);
    }
    *out_x1 = x1;
    *out_x2 = shifted_x2;
    *out_dist = -v3_length(v3_sub(x1, shifted_x2));
    return;
  }

  v3 p1 = pt->vert[2 * face[0]];
  v3 p2 = pt->vert[2 * face[1]];
  v3 p3 = pt->vert[2 * face[2]];
  v3 x1 = v3_make(p1.x * l1 + p2.x * l2 + p3.x * l3,
                  p1.y * l1 + p2.y * l2 + p3.y * l3,
                  p1.z * l1 + p2.z * l2 + p3.z * l3);
  *out_x1 = x1;
  *out_x2 = x2;
  *out_dist = -sqrtf(pt->face_norm2[face_idx]);
}
// G10@P5 G5：EPA polytope 初始构造设备版，与 host `gjk/polytope_build.rs` 逐式对应。
// Adapted from frozen MJWarp collision_gjk.py, revision 71da24d. Apache-2.0.
#pragma once

#define G5_MIN_DIST2 1.0e-10f
#define G5_MIN_DIST3 1.0e-10f
#define G5_MIN_DIST4 1.0e-17f

__device__ __forceinline__ void g5_next_face(d_polytope* pt, int idx1, int idx2, int idx3,
                                             d_gjk_result* next) {
  *next = g5_replace_simplex3(pt, idx1, idx2, idx3);
}

// 上游 `_polytope2`：由 GJK 的 1-单纯形构造 hexahedron。
__device__ __forceinline__ void g5_polytope2(
    d_polytope* pt, const v3* simplex, const v3* simplex1, const v3* simplex2,
    const int* simplex_index1, const int* simplex_index2, const d_geom& geom1,
    const d_geom& geom2, int geomtype1, int geomtype2, d_gjk_result* next) {
  v3 diff = v3_sub(simplex[1], simplex[0]);
  pt->center = v3_make(0.5f * (simplex[0].x + simplex[1].x),
                       0.5f * (simplex[0].y + simplex[1].y),
                       0.5f * (simplex[0].z + simplex[1].z));
  float value = G5_FLOAT_MAX;
  int index = 0;
  for (int i = 0; i < 3; ++i) {
    const float* d = reinterpret_cast<const float*>(&diff.x);
    if (fabsf(d[i]) < value) {
      value = fabsf(d[i]);
      index = i;
    }
  }
  v3 e = v3_make(0.0f, 0.0f, 0.0f);
  float* ep = reinterpret_cast<float*>(&e.x);
  ep[index] = 1.0f;
  v3 d1 = v3_cross(e, diff);
  float r[9];
  g5_rotmat(diff, r);
  v3 d2 = mul_mat3_vec(r, d1);
  v3 d3 = mul_mat3_vec(r, d2);

  pt->vert[0] = simplex1[0];
  pt->vert[1] = simplex2[0];
  pt->vert[2] = simplex1[1];
  pt->vert[3] = simplex2[1];
  pt->vert_index[0] = simplex_index1[0];
  pt->vert_index[1] = simplex_index2[0];
  pt->vert_index[2] = simplex_index1[1];
  pt->vert_index[3] = simplex_index2[1];

  int cache1 = 0;
  int cache2 = 0;
  float n1 = v3_length(d1);
  g5_epa_support(pt, 2, geom1, geom2, geomtype1, geomtype2,
                 v3_make(d1.x / n1, d1.y / n1, d1.z / n1), &cache1, &cache2);
  float n2 = v3_length(d2);
  g5_epa_support(pt, 3, geom1, geom2, geomtype1, geomtype2,
                 v3_make(d2.x / n2, d2.y / n2, d2.z / n2), &cache1, &cache2);
  float n3 = v3_length(d3);
  g5_epa_support(pt, 4, geom1, geom2, geomtype1, geomtype2,
                 v3_make(d3.x / n3, d3.y / n3, d3.z / n3), &cache1, &cache2);

  if (g5_attach_face(pt, 0, 0, 2, 3) < G5_MIN_DIST2) {
    pt->status = -1;
    g5_next_face(pt, 0, 2, 3, next);
    return;
  }
  if (g5_attach_face(pt, 1, 0, 4, 2) < G5_MIN_DIST2) {
    pt->status = -1;
    g5_next_face(pt, 0, 4, 2, next);
    return;
  }
  if (g5_attach_face(pt, 2, 0, 3, 4) < G5_MIN_DIST2) {
    pt->status = -1;
    g5_next_face(pt, 0, 3, 4, next);
    return;
  }
  if (g5_attach_face(pt, 3, 1, 3, 2) < G5_MIN_DIST2) {
    pt->status = -1;
    g5_next_face(pt, 1, 3, 2, next);
    return;
  }
  if (g5_attach_face(pt, 4, 1, 2, 4) < G5_MIN_DIST2) {
    pt->status = -1;
    g5_next_face(pt, 1, 2, 4, next);
    return;
  }
  if (g5_attach_face(pt, 5, 1, 4, 3) < G5_MIN_DIST2) {
    pt->status = -1;
    g5_next_face(pt, 1, 4, 3, next);
    return;
  }

  v3 v2 = v3_sub(pt->vert[4], pt->vert[5]);
  v3 v3v = v3_sub(pt->vert[6], pt->vert[7]);
  v3 v4 = v3_sub(pt->vert[8], pt->vert[9]);
  if (g5_ray_triangle(simplex[0], simplex[1], v2, v3v, v4) == 0) {
    pt->status = 1;
    *next = g5_gjk_result_zero();
    return;
  }
  pt->nvert = 5;
  pt->nface = 6;
  pt->status = 0;
  *next = g5_gjk_result_zero();
}

// 上游 `_polytope3`：由 GJK 的 2-单纯形构造 hexahedron。
__device__ __forceinline__ void g5_polytope3(
    d_polytope* pt, float dist, const v3* simplex, const v3* simplex1, const v3* simplex2,
    const int* simplex_index1, const int* simplex_index2, const d_geom& geom1,
    const d_geom& geom2, int geomtype1, int geomtype2) {
  v3 sum1 = v3_add(v3_add(simplex[0], simplex[1]), simplex[2]);
  pt->center = v3_make(sum1.x * (1.0f / 3.0f), sum1.y * (1.0f / 3.0f),
                       sum1.z * (1.0f / 3.0f));
  v3 n = v3_cross(v3_sub(simplex[1], simplex[0]), v3_sub(simplex[2], simplex[0]));
  float norm = v3_length(n);
  if (norm < G5_MINVAL) {
    pt->status = 2;
    return;
  }
  n = v3_make(n.x / norm, n.y / norm, n.z / norm);

  pt->vert[0] = simplex1[0];
  pt->vert[1] = simplex2[0];
  pt->vert[2] = simplex1[1];
  pt->vert[3] = simplex2[1];
  pt->vert[4] = simplex1[2];
  pt->vert[5] = simplex2[2];
  pt->vert_index[0] = simplex_index1[0];
  pt->vert_index[1] = simplex_index2[0];
  pt->vert_index[2] = simplex_index1[1];
  pt->vert_index[3] = simplex_index2[1];
  pt->vert_index[4] = simplex_index1[2];
  pt->vert_index[5] = simplex_index2[2];

  int cache1 = 0;
  int cache2 = 0;
  g5_epa_support(pt, 3, geom1, geom2, geomtype1, geomtype2, v3_neg(n), &cache1, &cache2);
  g5_epa_support(pt, 4, geom1, geom2, geomtype1, geomtype2, n, &cache1, &cache2);

  v3 v1 = simplex[0];
  v3 v2 = simplex[1];
  v3 v3v = simplex[2];
  v3 v4 = v3_sub(pt->vert[6], pt->vert[7]);
  v3 v5 = v3_sub(pt->vert[8], pt->vert[9]);
  if (g5_tri_point_intersect(v1, v2, v3v, v4)) {
    pt->status = 3;
    return;
  }
  if (g5_tri_point_intersect(v1, v2, v3v, v5)) {
    pt->status = 4;
    return;
  }
  if (dist > 1e-5f && !g5_test_tetra(v1, v2, v3v, v4) && !g5_test_tetra(v1, v2, v3v, v5)) {
    pt->status = 5;
    return;
  }
  if (g5_attach_face(pt, 0, 4, 0, 1) < G5_MIN_DIST3) {
    pt->status = 6;
    return;
  }
  if (g5_attach_face(pt, 1, 4, 2, 0) < G5_MIN_DIST3) {
    pt->status = 7;
    return;
  }
  if (g5_attach_face(pt, 2, 4, 1, 2) < G5_MIN_DIST3) {
    pt->status = 8;
    return;
  }
  if (g5_attach_face(pt, 3, 3, 1, 0) < G5_MIN_DIST3) {
    pt->status = 9;
    return;
  }
  if (g5_attach_face(pt, 4, 3, 0, 2) < G5_MIN_DIST3) {
    pt->status = 10;
    return;
  }
  if (g5_attach_face(pt, 5, 3, 2, 1) < G5_MIN_DIST3) {
    pt->status = 11;
    return;
  }
  pt->nvert = 5;
  pt->nface = 6;
  pt->status = 0;
}

// 上游 `_polytope4`：由 GJK 的 3-单纯形构造 tetrahedron。
__device__ __forceinline__ void g5_polytope4(d_polytope* pt, const v3* simplex,
                                             const v3* simplex1, const v3* simplex2,
                                             const int* simplex_index1,
                                             const int* simplex_index2,
                                             d_gjk_result* next) {
  v3 sum = v3_add(v3_add(simplex[0], simplex[1]), v3_add(simplex[2], simplex[3]));
  pt->center = v3_make(0.25f * sum.x, 0.25f * sum.y, 0.25f * sum.z);
  for (int i = 0; i < 4; ++i) {
    pt->vert[2 * i] = simplex1[i];
    pt->vert[2 * i + 1] = simplex2[i];
    pt->vert_index[2 * i] = simplex_index1[i];
    pt->vert_index[2 * i + 1] = simplex_index2[i];
  }
  float dist[4] = {0.0f, 0.0f, 0.0f, 0.0f};
  int idx = 0;
  dist[0] = g5_attach_face(pt, 0, 0, 1, 2);
  if (dist[0] < G5_MIN_DIST4) {
    pt->status = -1;
    g5_next_face(pt, 0, 1, 2, next);
    return;
  }
  dist[1] = g5_attach_face(pt, 1, 0, 3, 1);
  if (dist[1] < G5_MIN_DIST4) {
    pt->status = -1;
    g5_next_face(pt, 0, 3, 1, next);
    return;
  }
  idx = dist[0] < dist[1] ? 0 : 1;
  dist[2] = g5_attach_face(pt, 2, 0, 2, 3);
  if (dist[2] < G5_MIN_DIST4) {
    pt->status = -1;
    g5_next_face(pt, 0, 2, 3, next);
    return;
  }
  if (dist[2] < dist[idx]) idx = 2;
  dist[3] = g5_attach_face(pt, 3, 3, 2, 1);
  if (dist[3] < G5_MIN_DIST4) {
    pt->status = -1;
    g5_next_face(pt, 3, 2, 1, next);
    return;
  }
  if (dist[3] < dist[idx]) idx = 3;
  if (!g5_test_tetra(simplex[0], simplex[1], simplex[2], simplex[3])) {
    if (dist[idx] > G5_MINVAL) {
      pt->status = 12;
      *next = g5_gjk_result_zero();
      return;
    }
    pt->status = -1;
    if (idx == 0) {
      g5_next_face(pt, 0, 1, 2, next);
    } else if (idx == 1) {
      g5_next_face(pt, 0, 3, 1, next);
    } else if (idx == 2) {
      g5_next_face(pt, 0, 2, 3, next);
    } else {
      g5_next_face(pt, 3, 2, 1, next);
    }
    return;
  }
  pt->nvert = 4;
  pt->nface = 4;
  pt->status = 0;
  *next = g5_gjk_result_zero();
}
// G10@P5 G5：EPA 迭代设备版，与 host `gjk/epa.rs` 逐式对应。
// Adapted from frozen MJWarp collision_gjk.py, revision 71da24d. Apache-2.0.
#pragma once

#define G5_MIN_EPATOL 1.0e-7f
#define G5_OVERFLOW_EPA_HORIZON (1u << 8)

struct d_epa_result {
  float dist;
  v3 x1;
  v3 x2;
  int face_idx;
  unsigned overflow;
};

__device__ __forceinline__ void g5_get_edge(int edge, int* out) {
  out[0] = edge & 0x3FF;
  out[1] = (edge >> 10) & 0x3FF;
}

// 上游 `_add_edge`：重复边与末元素交换移除；返回新计数，-1 表示失败。
__device__ __forceinline__ int g5_add_edge(d_polytope* pt, int e1, int e2) {
  int n = pt->nhorizon;
  if (n < 0) return -1;
  int low = e1 < e2 ? e1 : e2;
  int high = e1 < e2 ? e2 : e1;
  int edge = (low << 10) | high;
  for (int i = 0; i < n; ++i) {
    if (edge == pt->horizon[i]) {
      pt->horizon[i] = pt->horizon[n - 1];
      return n - 1;
    }
  }
  if (n == pt->max_horizon) return -1;
  pt->horizon[n] = edge;
  return n + 1;
}

// 上游 `_epa()`：从初始 polytope 求穿透距离与 witness；polytope 原地更新。
__device__ __forceinline__ d_epa_result g5_epa(d_polytope* pt, float tolerance,
                                               int epa_iterations, const d_geom& geom1,
                                               const d_geom& geom2, int geomtype1,
                                               int geomtype2, bool is_discrete) {
  float upper = G5_FLOAT_MAX;
  float upper2 = G5_FLOAT_MAX;
  int idx = -1;
  int pidx = -1;
  unsigned overflow = 0u;
  float epsilon = is_discrete ? G5_MIN_EPATOL : tolerance;
  int nvalid = pt->nface;
  if (epa_iterations > 1000) epa_iterations = 1000;

  for (int iteration = 0; iteration < epa_iterations; ++iteration) {
    pidx = idx;
    idx = -1;
    float lower2 = G5_FLOAT_MAX;

    // 找离原点最近的面（穿透深度下界）。
    for (int i = 0; i < pt->nface; ++i) {
      if (!g5_is_invalid_face(pt->face[i]) && pt->face_norm2[i] < lower2) {
        idx = i;
        lower2 = pt->face_norm2[i];
      }
    }
    // 无有效面：回退到上一面。
    if (lower2 > upper2 || idx < 0) {
      idx = pidx;
      break;
    }
    if (lower2 <= 0.0f) break;

    // 由最近面的法线取支撑点 w。
    float lower = sqrtf(lower2);
    int wi = pt->nvert;
    v3 face_pr = pt->face_pr[idx];
    v3 dir = v3_make(face_pr.x / lower, face_pr.y / lower, face_pr.z / lower);
    int support_index1 = 0;
    int support_index2 = 0;
    g5_epa_support(pt, wi, geom1, geom2, geomtype1, geomtype2, dir, &support_index1,
                   &support_index2);
    v3 w = v3_sub(pt->vert[2 * wi], pt->vert[2 * wi + 1]);
    pt->nvert += 1;

    // 第 k 次迭代上界（归一化前点积，精度更好）。
    float upper_k = v3_dot(face_pr, w) / lower;
    if (upper_k < upper) {
      upper = upper_k;
      upper2 = upper * upper;
    }
    if (upper - lower < epsilon) break;

    // 检查 wi 是否为重复支撑点（离散几何提前终止）。
    if (is_discrete) {
      bool found_repeated = false;
      for (int i = 0; i < pt->nvert - 1; ++i) {
        if (pt->vert_index[2 * i] == pt->vert_index[2 * wi] &&
            pt->vert_index[2 * i + 1] == pt->vert_index[2 * wi + 1]) {
          found_repeated = true;
          break;
        }
      }
      if (found_repeated) break;
    }

    nvalid -= 1;
    pt->face[idx] = g5_delete_face(pt->face[idx]);
    int face[3];
    g5_get_face_verts(pt->face[idx], face);
    pt->nhorizon = g5_add_edge(pt, face[0], face[1]);
    pt->nhorizon = g5_add_edge(pt, face[1], face[2]);
    pt->nhorizon = g5_add_edge(pt, face[2], face[0]);
    if (pt->nhorizon == -1) {
      overflow |= G5_OVERFLOW_EPA_HORIZON;
      idx = -1;
      break;
    }

    // 计算 w 的 horizon。
    for (int i = 0; i < pt->nface; ++i) {
      if (g5_is_face_deleted(pt->face[i])) continue;
      if (v3_dot(pt->face_pr[i], w) - pt->face_norm2[i] > 1e-10f) {
        if (!g5_is_invalid_face(pt->face[i])) nvalid -= 1;
        pt->face[i] = g5_delete_face(pt->face[i]);
        int inner[3];
        g5_get_face_verts(pt->face[i], inner);
        pt->nhorizon = g5_add_edge(pt, inner[0], inner[1]);
        pt->nhorizon = g5_add_edge(pt, inner[1], inner[2]);
        pt->nhorizon = g5_add_edge(pt, inner[2], inner[0]);
        if (pt->nhorizon == -1) {
          overflow |= G5_OVERFLOW_EPA_HORIZON;
          idx = -1;
          break;
        }
      }
    }

    // 插入新顶点并沿 horizon 附面。
    int i = 0;
    while (i < pt->nhorizon) {
      int edge[2];
      g5_get_edge(pt->horizon[i], edge);
      int nface = pt->nface;
      float dist2 = g5_attach_face(pt, nface, wi, edge[0], edge[1]);
      if (dist2 == 0.0f) {
        idx = -1;
        break;
      }
      pt->nface += 1;
      if (dist2 >= lower2 && dist2 <= upper2) {
        nvalid += 1;
      } else {
        pt->face[pt->nface - 1] = g5_invalidate_face(pt->face[pt->nface - 1]);
      }
      i += 1;
    }
    if (nvalid == 0 || idx == -1) break;
    pt->nhorizon = 0;
  }

  d_epa_result result;
  result.overflow = overflow;
  if (idx > -1) {
    v3 x1;
    v3 x2;
    float dist;
    g5_epa_witness(pt, geom1, geom2, geomtype1, geomtype2, idx, &x1, &x2, &dist);
    result.dist = dist;
    result.x1 = x1;
    result.x2 = x2;
    result.face_idx = idx;
    return result;
  }
  result.dist = 0.0f;
  result.x1 = v3_make(0.0f, 0.0f, 0.0f);
  result.x2 = v3_make(0.0f, 0.0f, 0.0f);
  result.face_idx = -1;
  return result;
}
// G10@P5 G5：ccd 两阶段驱动设备版，与 host `gjk/phase.rs` + `gjk/ccd.rs` 逐式对应。
// Adapted from frozen MJWarp collision_gjk.py, revision 71da24d. Apache-2.0.
#pragma once

#define G5_OVERFLOW_CCD (1u << 4)

struct d_ccd_options {
  float tolerance;
  float cutoff;
  int gjk_iterations;
  int epa_iterations;
};

struct d_ccd_result {
  float dist;
  int ncontact;
  v3 x1;
  v3 x2;
  int multiccd_idx;
  unsigned overflow;
};

struct d_gjk_phase {
  bool needs_epa;
  float dist;
  int ncontact;
  v3 x1;
  v3 x2;
  d_gjk_result result;
  d_geom geom1;
  d_geom geom2;
};

struct d_epa_phase {
  float dist;
  int ncontact;
  v3 x1;
  v3 x2;
  int multiccd_idx;
  bool has_polytope;
  unsigned overflow;
};

// 上游 `_inflate`：浅穿透把 witness 沿法线膨胀回真实表面。
__device__ __forceinline__ void g5_inflate(const d_gjk_result* result, const d_geom& geom1,
                                           const d_geom& geom2, int geomtype1,
                                           int geomtype2, float margin1, float margin2,
                                           float* out_dist, v3* out_x1, v3* out_x2) {
  float dist = result->dist;
  v3 x1 = result->x1;
  v3 x2 = result->x2;
  if (geomtype1 == G5_HFIELD) {
    int v = result->simplex_index1[0];
    bool is_side = false;
    for (int i = 0; i < result->dim; ++i) {
      if (result->simplex_index1[i] != v) {
        is_side = true;
        break;
      }
    }
    if (is_side) {
      v3 n = v3_make(0.0f, 0.0f, 1.0f);
      d_support_point sp = support(geom2, geomtype2, x2);
      x2 = v3_sub(sp.point, v3_scale(n, margin2));
      v3 a = v3_make(geom1.polyvert[9], geom1.polyvert[10], geom1.polyvert[11]);
      v3 b = v3_make(geom1.polyvert[12], geom1.polyvert[13], geom1.polyvert[14]);
      v3 c = v3_make(geom1.polyvert[15], geom1.polyvert[16], geom1.polyvert[17]);
      float coordinates[3];
      g5_tri_affine_coord(a, b, c, x2, coordinates);
      if (coordinates[0] > 0.0f && coordinates[1] > 0.0f && coordinates[2] > 0.0f) {
        x1 = v3_make(coordinates[0] * a.x + coordinates[1] * b.x + coordinates[2] * c.x,
                     coordinates[0] * a.y + coordinates[1] * b.y + coordinates[2] * c.y,
                     coordinates[0] * a.z + coordinates[1] * b.z + coordinates[2] * c.z);
      } else {
        v3 p = c;
        if (coordinates[1] > 0.0f) p = b;
        if (coordinates[0] > 0.0f) p = a;
        float d = v3_dot(v3_sub(x2, p), n);
        x1 = v3_make(x2.x - d * n.x, x2.y - d * n.y, x2.z - d * n.z);
      }
      dist = -v3_length(v3_sub(x1, x2));
      *out_dist = dist;
      *out_x1 = x1;
      *out_x2 = x2;
      return;
    }
  }
  v3 n = v3_normalize(v3_sub(x2, x1));
  if (margin1 > 0.0f) x1 = v3_add(x1, v3_scale(n, margin1));
  if (margin2 > 0.0f) x2 = v3_sub(x2, v3_scale(n, margin2));
  dist -= margin1 + margin2;
  *out_dist = dist;
  *out_x1 = x1;
  *out_x2 = x2;
}

// 上游 `gjk_phase`。
__device__ __forceinline__ void g5_gjk_phase(float tolerance, float cutoff,
                                             int gjk_iterations, d_geom geom1, d_geom geom2,
                                             int geomtype1, int geomtype2, v3 x_1, v3 x_2,
                                             d_gjk_phase* out) {
  float orig_margin1 = geom1.margin;
  float orig_margin2 = geom2.margin;
  v3 orig_size1 = geom1.size;
  v3 orig_size2 = geom2.size;
  float full_margin1 = 0.0f;
  float full_margin2 = 0.0f;
  float size1 = 0.0f;
  float size2 = 0.0f;
  d_gjk_result empty = g5_gjk_result_zero();
  bool is_discrete = g5_is_discrete(geomtype1, geomtype2, geom1.margin, geom2.margin);

  if (geomtype1 == G5_SPHERE || geomtype1 == G5_CAPSULE) {
    size1 = geom1.size.x;
    full_margin1 = size1 + 0.5f * orig_margin1;
    geom1.margin = 0.0f;
    geom1.size = v3_make(0.0f, geom1.size.y, geom1.size.z);
  }
  if (geomtype2 == G5_SPHERE || geomtype2 == G5_CAPSULE) {
    size2 = geom2.size.x;
    full_margin2 = size2 + 0.5f * orig_margin2;
    geom2.margin = 0.0f;
    geom2.size = v3_make(0.0f, geom2.size.y, geom2.size.z);
  }

  if (size1 + size2 > 0.0f) {
    cutoff += full_margin1 + full_margin2;
    d_gjk_result result =
        g5_gjk(tolerance, gjk_iterations, geom1, geom2, x_1, x_2, geomtype1, geomtype2,
               cutoff, is_discrete);
    geom1.index = result.index1;
    geom2.index = result.index2;
    if (result.dist > tolerance) {
      out->needs_epa = false;
      out->ncontact = 1;
      out->result = empty;
      out->geom1 = geom1;
      out->geom2 = geom2;
      if (result.dist == G5_FLOAT_MAX) {
        out->dist = result.dist;
        out->x1 = result.x1;
        out->x2 = result.x2;
        return;
      }
      g5_inflate(&result, geom1, geom2, geomtype1, geomtype2, full_margin1, full_margin2,
                 &out->dist, &out->x1, &out->x2);
      return;
    }
    geom1.margin = orig_margin1;
    geom1.size = orig_size1;
    geom2.margin = orig_margin2;
    geom2.size = orig_size2;
    cutoff -= full_margin1 + full_margin2;
  }

  d_gjk_result result =
      g5_gjk(tolerance, gjk_iterations, geom1, geom2, x_1, x_2, geomtype1, geomtype2, cutoff,
             is_discrete);
  geom1.index = result.index1;
  geom2.index = result.index2;
  out->geom1 = geom1;
  out->geom2 = geom2;
  if (result.dist > tolerance || result.dim < 2 || result.separated) {
    out->needs_epa = false;
    out->dist = result.dist;
    out->ncontact = 1;
    out->x1 = result.x1;
    out->x2 = result.x2;
    out->result = empty;
    return;
  }
  out->needs_epa = true;
  out->dist = result.dist;
  out->ncontact = 1;
  out->x1 = result.x1;
  out->x2 = result.x2;
  out->result = result;
}

// 上游 `epa_phase`：调用方提供已清零计数的 polytope 缓冲。
__device__ __forceinline__ void g5_epa_phase(float tolerance, int epa_iterations,
                                             d_gjk_result result, const d_geom& geom1,
                                             const d_geom& geom2, int geomtype1,
                                             int geomtype2, d_polytope* pt,
                                             d_epa_phase* out) {
  d_gjk_result next = g5_gjk_result_zero();
  pt->nvert = 0;
  pt->nface = 0;
  pt->nhorizon = 0;
  pt->status = 0;
  pt->horizon[0] = 0;
  if (result.dim == 2) {
    g5_polytope2(pt, result.simplex, result.simplex1, result.simplex2, result.simplex_index1,
                 result.simplex_index2, geom1, geom2, geomtype1, geomtype2, &next);
    if (pt->status == -1) {
      result.simplex[0] = next.simplex[0];
      result.simplex[1] = next.simplex[1];
      result.simplex[2] = next.simplex[2];
      result.simplex1[0] = next.simplex1[0];
      result.simplex1[1] = next.simplex1[1];
      result.simplex1[2] = next.simplex1[2];
      result.simplex2[0] = next.simplex2[0];
      result.simplex2[1] = next.simplex2[1];
      result.simplex2[2] = next.simplex2[2];
      result.simplex_index1[0] = next.simplex_index1[0];
      result.simplex_index1[1] = next.simplex_index1[1];
      result.simplex_index1[2] = next.simplex_index1[2];
      result.simplex_index2[0] = next.simplex_index2[0];
      result.simplex_index2[1] = next.simplex_index2[1];
      result.simplex_index2[2] = next.simplex_index2[2];
      result.dim = 3;
    }
  } else if (result.dim == 4) {
    g5_polytope4(pt, result.simplex, result.simplex1, result.simplex2, result.simplex_index1,
                 result.simplex_index2, &next);
    if (pt->status == -1) {
      result.simplex[0] = next.simplex[0];
      result.simplex[1] = next.simplex[1];
      result.simplex[2] = next.simplex[2];
      result.simplex1[0] = next.simplex1[0];
      result.simplex1[1] = next.simplex1[1];
      result.simplex1[2] = next.simplex1[2];
      result.simplex2[0] = next.simplex2[0];
      result.simplex2[1] = next.simplex2[1];
      result.simplex2[2] = next.simplex2[2];
      result.simplex_index1[0] = next.simplex_index1[0];
      result.simplex_index1[1] = next.simplex_index1[1];
      result.simplex_index1[2] = next.simplex_index1[2];
      result.simplex_index2[0] = next.simplex_index2[0];
      result.simplex_index2[1] = next.simplex_index2[1];
      result.simplex_index2[2] = next.simplex_index2[2];
      result.dim = 3;
    }
  }
  if (result.dim == 3) {
    g5_polytope3(pt, result.dist, result.simplex, result.simplex1, result.simplex2,
                 result.simplex_index1, result.simplex_index2, geom1, geom2, geomtype1,
                 geomtype2);
  }

  // 原点在边界上：不算穿透。
  if (pt->status != 0) {
    out->dist = result.dist;
    out->ncontact = 1;
    out->x1 = result.x1;
    out->x2 = result.x2;
    out->multiccd_idx = -1;
    out->has_polytope = false;
    out->overflow = 0;
    return;
  }

  bool is_discrete = g5_is_discrete(geomtype1, geomtype2, geom1.margin, geom2.margin);
  d_epa_result epa_result =
      g5_epa(pt, tolerance, epa_iterations, geom1, geom2, geomtype1, geomtype2, is_discrete);
  if (epa_result.face_idx == -1) {
    out->dist = G5_FLOAT_MAX;
    out->ncontact = 0;
    out->x1 = v3_make(0.0f, 0.0f, 0.0f);
    out->x2 = v3_make(0.0f, 0.0f, 0.0f);
    out->multiccd_idx = -1;
    out->has_polytope = false;
    out->overflow = epa_result.overflow;
    return;
  }
  int multiccd_idx = epa_result.face_idx;
  if (geom1.margin != 0.0f || geom2.margin != 0.0f) multiccd_idx = -1;
  bool supported = (geomtype1 == G5_BOX || geomtype1 == G5_MESH ||
                    geomtype1 == G5_CYLINDER) &&
                   (geomtype2 == G5_BOX || geomtype2 == G5_MESH ||
                    geomtype2 == G5_CYLINDER);
  if (!supported) multiccd_idx = -1;
  out->dist = epa_result.dist;
  out->ncontact = 1;
  out->x1 = epa_result.x1;
  out->x2 = epa_result.x2;
  out->multiccd_idx = multiccd_idx;
  out->has_polytope = true;
  out->overflow = epa_result.overflow;
}

// 上游 `ccd()`：通用凸碰撞入口；polytope 缓冲由调用方按 epa_iterations 分配。
__device__ __forceinline__ void g5_ccd(d_geom geom1, d_geom geom2, int geomtype1,
                                       int geomtype2, v3 x_1, v3 x_2, d_ccd_options options,
                                       d_polytope* pt, d_ccd_result* out) {
  d_gjk_phase phase;
  g5_gjk_phase(options.tolerance, options.cutoff, options.gjk_iterations, geom1, geom2,
               geomtype1, geomtype2, x_1, x_2, &phase);
  if (!phase.needs_epa) {
    out->dist = phase.dist;
    out->ncontact = phase.ncontact;
    out->x1 = phase.x1;
    out->x2 = phase.x2;
    out->multiccd_idx = -1;
    out->overflow = 0;
    return;
  }
  d_epa_phase epa_phase_result;
  g5_epa_phase(options.tolerance, options.epa_iterations, phase.result, phase.geom1,
               phase.geom2, geomtype1, geomtype2, pt, &epa_phase_result);
  out->dist = epa_phase_result.dist;
  out->ncontact = epa_phase_result.ncontact;
  out->x1 = epa_phase_result.x1;
  out->x2 = epa_phase_result.x2;
  out->multiccd_idx = epa_phase_result.multiccd_idx;
  // 上游 `ccd()` 只透传 EPA 阶段产生的溢出位（`EPA_HORIZON`）；`OverflowType.CCD`
  // 只由调用方在 `nccd >= naccdmax` 的真实容量截断时置（见 contact.cu 的容量门），
  // 不由 EPA 返回 `FLOAT_MAX`（horizon 溢出/退化）推断。
  out->overflow = epa_phase_result.overflow;
}
// G10@P5 G5：SDF octree 采样设备版，与 host `sdf/octree.rs` 逐式对应。
// Adapted from frozen MJWarp collision_sdf.py, revision 71da24d. Apache-2.0.
#pragma once

// 体积 SDF 数据（G1 字段上传口径：oct_aabb/oct_child/oct_coeff/mesh_octadr）。
struct d_volume_data {
  v3 center;
  v3 half_size;
  // 每节点 6 个 float：(center.xyz, half_size.xyz)。
  const float* oct_aabb;
  // 每节点 8 个 int（相对根；-1 = 叶）。
  const int* oct_child;
  // 每节点 8 个 float 插值系数。
  const float* oct_coeff;
  int root;
  int noct;
};

struct d_oct_hit {
  bool ok;
  int node;
  float rx[8];
  float ry[8];
  float rz[8];
};

__device__ __forceinline__ float g5_dot8(const float* a, const float* b) {
  float sum = 0.0f;
  for (int index = 0; index < 8; ++index) sum += a[index] * b[index];
  return sum;
}

// 上游 `box_project`：返盒距与投影点（外扩 eps=1e-4）。
__device__ __forceinline__ float g5_box_project(v3 center, v3 half_size, v3 xyz, v3* out_point) {
  v3 r = v3_sub(xyz, center);
  float qx = fabsf(r.x) - half_size.x;
  float qy = fabsf(r.y) - half_size.y;
  float qz = fabsf(r.z) - half_size.z;
  if (qx <= 0.0f && qy <= 0.0f && qz <= 0.0f) {
    *out_point = xyz;
    return 0.0f;
  }
  float dist_sqr = 0.0f;
  const float eps = 1.0e-4f;
  v3 point = xyz;
  if (qx >= 0.0f) {
    dist_sqr += qx * qx;
    point.x = r.x > 0.0f ? point.x - (qx + eps) : point.x + (qx + eps);
  }
  if (qy >= 0.0f) {
    dist_sqr += qy * qy;
    point.y = r.y > 0.0f ? point.y - (qy + eps) : point.y + (qy + eps);
  }
  if (qz >= 0.0f) {
    dist_sqr += qz * qz;
    point.z = r.z > 0.0f ? point.z - (qz + eps) : point.z + (qz + eps);
  }
  *out_point = point;
  return sqrtf(dist_sqr);
}

// 上游 `find_oct`：从根按坐标子索引下降到叶，返回叶节点与三轴权重。
__device__ __forceinline__ d_oct_hit g5_find_oct(const d_volume_data* volume, v3 p, bool grad) {
  d_oct_hit hit;
  hit.ok = false;
  hit.node = -1;
  for (int i = 0; i < 8; ++i) {
    hit.rx[i] = 0.0f;
    hit.ry[i] = 0.0f;
    hit.rz[i] = 0.0f;
  }
  long long stack = volume->root;
  int niter = 100;
  const float eps = 1.0e-6f;
  while (niter > 0) {
    niter -= 1;
    long long node = stack;
    if (node < 0) return hit;
    const float* aabb = volume->oct_aabb + node * 6;
    v3 center = v3_make(aabb[0], aabb[1], aabb[2]);
    v3 half = v3_make(aabb[3], aabb[4], aabb[5]);
    v3 vmin = v3_sub(center, half);
    v3 vmax = v3_add(center, half);
    if (p.x + eps < vmin.x || p.x - eps > vmax.x || p.y + eps < vmin.y ||
        p.y - eps > vmax.y || p.z + eps < vmin.z || p.z - eps > vmax.z) {
      continue;
    }
    v3 coord = v3_make((p.x - vmin.x) / (vmax.x - vmin.x), (p.y - vmin.y) / (vmax.y - vmin.y),
                       (p.z - vmin.z) / (vmax.z - vmin.z));
    const int* child = volume->oct_child + node * 8;
    bool leaf = true;
    for (int i = 0; i < 8; ++i) {
      if (child[i] != -1) {
        leaf = false;
        break;
      }
    }
    if (leaf) {
      if (!grad) {
        for (int j = 0; j < 8; ++j) {
          float x = (j & 1) != 0 ? coord.x : 1.0f - coord.x;
          float y = (j & 2) != 0 ? coord.y : 1.0f - coord.y;
          float z = (j & 4) != 0 ? coord.z : 1.0f - coord.z;
          hit.rx[j] = x * y * z;
        }
      } else {
        v3 inv_cell = v3_make(1.0f / (vmax.x - vmin.x), 1.0f / (vmax.y - vmin.y),
                              1.0f / (vmax.z - vmin.z));
        for (int j = 0; j < 8; ++j) {
          float x = (j & 1) != 0 ? coord.x : 1.0f - coord.x;
          float y = (j & 2) != 0 ? coord.y : 1.0f - coord.y;
          float z = (j & 4) != 0 ? coord.z : 1.0f - coord.z;
          float sx = (j & 1) != 0 ? 1.0f : -1.0f;
          float sy = (j & 2) != 0 ? 1.0f : -1.0f;
          float sz = (j & 4) != 0 ? 1.0f : -1.0f;
          hit.rx[j] = sx * y * z * inv_cell.x;
          hit.ry[j] = x * sy * z * inv_cell.y;
          hit.rz[j] = x * y * sz * inv_cell.z;
        }
      }
      hit.ok = true;
      hit.node = (int)node;
      return hit;
    }
    int x = coord.x < 0.5f ? 0 : 1;
    int y = coord.y < 0.5f ? 0 : 1;
    int z = coord.z < 0.5f ? 0 : 1;
    int next = child[4 * z + 2 * y + x];
    stack = next != -1 ? (long long)next + volume->root : -1;
  }
  return hit;
}

// 上游 `sample_volume_sdf`：盒距 + 叶节点三线性插值系数点积。
__device__ __forceinline__ bool g5_sample_volume_sdf(const d_volume_data* volume, v3 xyz,
                                                     float* out) {
  v3 point;
  float dist0 = g5_box_project(volume->center, volume->half_size, xyz, &point);
  d_oct_hit hit = g5_find_oct(volume, point, false);
  if (!hit.ok) return false;
  *out = dist0 + g5_dot8(hit.rx, volume->oct_coeff + hit.node * 8);
  return true;
}

// 上游 `sample_volume_grad`：盒外有限差分（h=1e-4），盒内解析权重梯度。
__device__ __forceinline__ bool g5_sample_volume_grad(const d_volume_data* volume, v3 xyz,
                                                      v3* out) {
  v3 point;
  float dist0 = g5_box_project(volume->center, volume->half_size, xyz, &point);
  if (dist0 > 0.0f) {
    const float h = 1.0e-4f;
    float f = 0.0f;
    if (!g5_sample_volume_sdf(volume, xyz, &f)) return false;
    float fx = 0.0f;
    float fy = 0.0f;
    float fz = 0.0f;
    if (!g5_sample_volume_sdf(volume, v3_make(xyz.x + h, xyz.y, xyz.z), &fx)) return false;
    if (!g5_sample_volume_sdf(volume, v3_make(xyz.x, xyz.y + h, xyz.z), &fy)) return false;
    if (!g5_sample_volume_sdf(volume, v3_make(xyz.x, xyz.y, xyz.z + h), &fz)) return false;
    *out = v3_make((fx - f) / h, (fy - f) / h, (fz - f) / h);
    return true;
  }
  d_oct_hit hit = g5_find_oct(volume, point, true);
  if (!hit.ok) return false;
  const float* coeff = volume->oct_coeff + hit.node * 8;
  *out = v3_make(g5_dot8(hit.rx, coeff), g5_dot8(hit.ry, coeff), g5_dot8(hit.rz, coeff));
  return true;
}

// 上游 `halton`：van der Corput 基数序列（f32 累加顺序与上游一致）。
__device__ __forceinline__ float g5_halton(unsigned index, unsigned base) {
  unsigned n0 = index;
  float b = (float)base;
  float f = 1.0f / b;
  float hn = 0.0f;
  while (n0 > 0) {
    unsigned n1 = n0 / base;
    unsigned r = n0 - n1 * base;
    hn += f * (float)r;
    f /= b;
    n0 = n1;
  }
  return hn;
}

// 上游 `transform_aabb`：局部 AABB 变换后取包围盒（min/max 输出）。
__device__ __forceinline__ void g5_transform_aabb(v3 center, v3 half_size, v3 pos,
                                                  const float* ori, v3* out_min,
                                                  v3* out_max) {
  v3 min_v = v3_make(1.0e9f, 1.0e9f, 1.0e9f);
  v3 max_v = v3_make(-1.0e9f, -1.0e9f, -1.0e9f);
  for (int index = 0; index < 8; ++index) {
    v3 corner = v3_make(half_size.x * ((index & 1) != 0 ? 1.0f : -1.0f),
                        half_size.y * ((index & 2) != 0 ? 1.0f : -1.0f),
                        half_size.z * ((index & 4) != 0 ? 1.0f : -1.0f));
    v3 frame = v3_add(mul_mat3_vec(ori, v3_add(corner, center)), pos);
    min_v.x = fminf(min_v.x, frame.x);
    min_v.y = fminf(min_v.y, frame.y);
    min_v.z = fminf(min_v.z, frame.z);
    max_v.x = fmaxf(max_v.x, frame.x);
    max_v.y = fmaxf(max_v.y, frame.y);
    max_v.z = fmaxf(max_v.z, frame.z);
  }
  *out_min = min_v;
  *out_max = max_v;
}
// G10@P5 G5：SDF 数学设备版，与 host `sdf/primitives.rs` + `sdf/descent.rs` 逐式对应。
// Adapted from frozen MJWarp collision_sdf.py, revision 71da24d. Apache-2.0.
// 注意：rel_mat 一律按上游 `transpose(rot1) * rot2`（common.cu 的 relative_matrix 已按上游语义）。
#pragma once

#define G5_MINVAL 1.0e-15f

// SDF 几何类型（与 host `SdfType` 一一对应）。
#define G5_SDF_PLANE 0
#define G5_SDF_SPHERE 1
#define G5_SDF_CAPSULE 2
#define G5_SDF_CYLINDER 3
#define G5_SDF_BOX 4
#define G5_SDF_ELLIPSOID 5
#define G5_SDF_MESH 6
#define G5_SDF_SDF 7

struct d_sdf_shape {
  int geom;
  v3 size;
  const d_volume_data* volume;
  bool has_volume;
};

struct d_opt_params {
  float rel_mat[9];
  v3 rel_pos;
  v3 attr1;
  v3 attr2;
};

// ---- 原语 SDF 与梯度（与 host `sdf/primitives.rs` 一致） ----

__device__ __forceinline__ v3 g5_abs3(v3 a) {
  return v3_make(fabsf(a.x), fabsf(a.y), fabsf(a.z));
}

__device__ __forceinline__ v3 g5_cw_div3(v3 a, v3 b) {
  return v3_make(a.x / b.x, a.y / b.y, a.z / b.z);
}

__device__ __forceinline__ v3 g5_max3(v3 a, v3 b) {
  return v3_make(fmaxf(a.x, b.x), fmaxf(a.y, b.y), fmaxf(a.z, b.z));
}

__device__ __forceinline__ v3 g5_radial_field(v3 a, v3 x, v3 size) {
  v3 field = v3_normalize(g5_cw_div3(v3_neg(size), a));
  // 上游 `wp.sign` 语义：`x < 0 ? -1 : 1`（零/-0.0 → +1，不返回 0）。
  return v3_make(field.x * (x.x < 0.0f ? -1.0f : 1.0f),
                 field.y * (x.y < 0.0f ? -1.0f : 1.0f),
                 field.z * (x.z < 0.0f ? -1.0f : 1.0f));
}

__device__ __forceinline__ float g5_prim_sphere(v3 p, v3 size) {
  return v3_length(p) - size.x;
}

__device__ __forceinline__ float g5_prim_box(v3 p, v3 size) {
  v3 a = v3_sub(g5_abs3(p), size);
  if (a.x >= 0.0f || a.y >= 0.0f || a.z >= 0.0f) {
    v3 b = g5_max3(a, v3_make(0.0f, 0.0f, 0.0f));
    return v3_length(b) + fminf(fmaxf(a.x, fmaxf(a.y, a.z)), 0.0f);
  }
  v3 b = g5_radial_field(a, p, size);
  v3 t = g5_cw_div3(v3_neg(a), g5_abs3(b));
  return -fminf(t.x, fminf(t.y, t.z)) * v3_length(b);
}

__device__ __forceinline__ float g5_prim_ellipsoid(v3 p, v3 size) {
  v3 scaled = v3_make(p.x / size.x, p.y / size.y, p.z / size.z);
  float k0 = v3_length(scaled);
  float k1 = v3_length(v3_make(p.x / (size.x * size.x), p.y / (size.y * size.y),
                               p.z / (size.z * size.z)));
  float denom = k1 != 0.0f ? k1 : 1.0e-12f;
  return k0 * (k0 - 1.0f) / denom;
}

__device__ __forceinline__ float g5_prim_capsule(v3 p, v3 size) {
  float r = size.x;
  float h = size.y;
  float pz = fminf(fmaxf(p.z, -h), h);
  return v3_length(v3_make(p.x, p.y, p.z - pz)) - r;
}

__device__ __forceinline__ float g5_prim_cylinder(v3 p, v3 size) {
  float r = size.x;
  float h = size.y;
  float dx = v3_length(v3_make(p.x, p.y, 0.0f)) - r;
  float dy = fabsf(p.z) - h;
  return fminf(fmaxf(dx, dy), 0.0f) +
         v3_length(v3_make(fmaxf(dx, 0.0f), fmaxf(dy, 0.0f), 0.0f));
}

__device__ __forceinline__ v3 g5_grad_sphere(v3 p) {
  float c = v3_length(p);
  if (c > 1.0e-9f) return v3_make(p.x / c, p.y / c, p.z / c);
  return v3_make(0.0f, 0.0f, 0.0f);
}

__device__ __forceinline__ v3 g5_grad_box(v3 p, v3 size) {
  v3 a = v3_sub(g5_abs3(p), size);
  if (fmaxf(a.x, fmaxf(a.y, a.z)) < 0.0f) return g5_radial_field(a, p, size);
  v3 b = g5_max3(a, v3_make(0.0f, 0.0f, 0.0f));
  float c = v3_length(b);
  v3 g = v3_make(b.x / c * (p.x / fabsf(p.x)), b.y / c * (p.y / fabsf(p.y)),
                 b.z / c * (p.z / fabsf(p.z)));
  if (a.x <= 0.0f) g.x = 0.0f;
  if (a.y <= 0.0f) g.y = 0.0f;
  if (a.z <= 0.0f) g.z = 0.0f;
  return g;
}

__device__ __forceinline__ v3 g5_grad_ellipsoid(v3 p, v3 size) {
  v3 a = v3_make(p.x / size.x, p.y / size.y, p.z / size.z);
  v3 b = v3_make(a.x / size.x, a.y / size.y, a.z / size.z);
  float k0 = v3_length(a);
  float k1 = v3_length(b);
  float inv_k0 = 1.0f / k0;
  float inv_k1 = 1.0f / k1;
  v3 gk0 = v3_make(b.x * inv_k0, b.y * inv_k0, b.z * inv_k0);
  v3 gk1 = v3_make(b.x * inv_k1 / (size.x * size.x), b.y * inv_k1 / (size.y * size.y),
                   b.z * inv_k1 / (size.z * size.z));
  float df_dk0 = (2.0f * k0 - 1.0f) * inv_k1;
  float df_dk1 = k0 * (k0 - 1.0f) * inv_k1 * inv_k1;
  v3 raw = v3_make(gk0.x * df_dk0 - gk1.x * df_dk1, gk0.y * df_dk0 - gk1.y * df_dk1,
                   gk0.z * df_dk0 - gk1.z * df_dk1);
  return v3_normalize(raw);
}

__device__ __forceinline__ v3 g5_grad_capsule(v3 p, v3 size) {
  float h = size.y;
  float pz = fminf(fmaxf(p.z, -h), h);
  v3 diff = v3_make(p.x, p.y, p.z - pz);
  float c = v3_length(diff);
  if (c > G5_MINVAL) return v3_make(diff.x / c, diff.y / c, diff.z / c);
  return v3_make(0.0f, 0.0f, 0.0f);
}

__device__ __forceinline__ v3 g5_grad_cylinder(v3 p, v3 size) {
  float r = size.x;
  float h = size.y;
  float radial_dist = v3_length(v3_make(p.x, p.y, 0.0f));
  v3 u = radial_dist > G5_MINVAL
             ? v3_make(p.x / radial_dist, p.y / radial_dist, 0.0f)
             : v3_make(0.0f, 0.0f, 0.0f);
  // 上游 `wp.sign(p[2])`（零/-0.0 → +1）。
  v3 w = v3_make(0.0f, 0.0f, p.z < 0.0f ? -1.0f : 1.0f);
  float dx = radial_dist - r;
  float dy = fabsf(p.z) - h;
  if (dx > 0.0f && dy > 0.0f) {
    float len_v = v3_length(v3_make(dx, dy, 0.0f));
    if (len_v > G5_MINVAL) {
      return v3_make(u.x * (dx / len_v) + w.x * (dy / len_v),
                     u.y * (dx / len_v) + w.y * (dy / len_v),
                     u.z * (dx / len_v) + w.z * (dy / len_v));
    }
    return v3_make(0.0f, 0.0f, 0.0f);
  }
  if (dx > 0.0f) return u;
  if (dy > 0.0f) return w;
  return dx > dy ? u : w;
}

// ---- sdf / sdf_grad 分发 ----

__device__ __forceinline__ bool g5_sdf_volume(const d_sdf_shape* shape, v3 p, float* out) {
  if (!shape->has_volume) return false;
  return g5_sample_volume_sdf(shape->volume, p, out);
}

__device__ __forceinline__ bool g5_sdf_volume_grad(const d_sdf_shape* shape, v3 p, v3* out) {
  if (!shape->has_volume) return false;
  return g5_sample_volume_grad(shape->volume, p, out);
}

__device__ __forceinline__ bool g5_sdf(const d_sdf_shape* shape, v3 p, float* out) {
  switch (shape->geom) {
    case G5_SDF_PLANE: *out = p.z; return true;
    case G5_SDF_SPHERE: *out = g5_prim_sphere(p, shape->size); return true;
    case G5_SDF_CAPSULE: *out = g5_prim_capsule(p, shape->size); return true;
    case G5_SDF_CYLINDER: *out = g5_prim_cylinder(p, shape->size); return true;
    case G5_SDF_BOX: *out = g5_prim_box(p, shape->size); return true;
    case G5_SDF_ELLIPSOID: *out = g5_prim_ellipsoid(p, shape->size); return true;
    case G5_SDF_MESH:
    case G5_SDF_SDF: return g5_sdf_volume(shape, p, out);
    default: return false;
  }
}

__device__ __forceinline__ bool g5_sdf_grad(const d_sdf_shape* shape, v3 p, v3* out) {
  switch (shape->geom) {
    case G5_SDF_PLANE: *out = v3_make(0.0f, 0.0f, 1.0f); return true;
    case G5_SDF_SPHERE: *out = g5_grad_sphere(p); return true;
    case G5_SDF_CAPSULE: *out = g5_grad_capsule(p, shape->size); return true;
    case G5_SDF_CYLINDER: *out = g5_grad_cylinder(p, shape->size); return true;
    case G5_SDF_BOX: *out = g5_grad_box(p, shape->size); return true;
    case G5_SDF_ELLIPSOID: *out = g5_grad_ellipsoid(p, shape->size); return true;
    case G5_SDF_MESH:
    case G5_SDF_SDF: return g5_sdf_volume_grad(shape, p, out);
    default: return false;
  }
}

// 上游对 geom2 强制 `GeomType.SDF` 的调用。
__device__ __forceinline__ bool g5_forced_sdf(const d_sdf_shape* shape, v3 p, float* out) {
  return g5_sdf_volume(shape, p, out);
}

__device__ __forceinline__ bool g5_forced_sdf_grad(const d_sdf_shape* shape, v3 p, v3* out) {
  return g5_sdf_volume_grad(shape, p, out);
}

// 上游 `clearance`。
__device__ __forceinline__ float g5_clearance(const d_sdf_shape* shape1,
                                              const d_sdf_shape* shape2, v3 p1, v3 p2,
                                              bool sfd_intersection) {
  float sdf1 = 0.0f;
  float sdf2 = 0.0f;
  g5_sdf(shape1, p1, &sdf1);
  g5_forced_sdf(shape2, p2, &sdf2);
  float max_val = fmaxf(sdf1, sdf2);
  return sfd_intersection ? max_val : sdf1 + sdf2 + fabsf(max_val);
}

// 上游 `compute_grad`。
__device__ __forceinline__ v3 g5_compute_grad(const d_sdf_shape* shape1,
                                              const d_sdf_shape* shape2, v3 p1, v3 p2,
                                              const d_opt_params* params,
                                              bool sfd_intersection) {
  float a_ = 0.0f;
  float b_ = 0.0f;
  v3 grad1 = v3_make(0.0f, 0.0f, 0.0f);
  v3 grad2 = v3_make(0.0f, 0.0f, 0.0f);
  g5_sdf(shape1, p1, &a_);
  g5_forced_sdf(shape2, p2, &b_);
  g5_sdf_grad(shape1, p1, &grad1);
  g5_forced_sdf_grad(shape2, p2, &grad2);
  v3 grad1_transformed = transpose_mul_mat3_vec(params->rel_mat, grad1);
  if (sfd_intersection) return a_ > b_ ? grad1_transformed : grad2;
  v3 gradient = v3_add(grad2, grad1_transformed);
  float max_val = fmaxf(a_, b_);
  v3 max_grad = a_ > b_ ? grad1_transformed : grad2;
  // 上游 `wp.sign(max_val)`（零/-0.0 → +1）。
  float sign = max_val < 0.0f ? -1.0f : 1.0f;
  return v3_add(gradient, v3_scale(max_grad, sign));
}

// 上游 `gradient_step`：Armijo 回溯（amin=1e-4、rho=0.5、c=0.1）。
__device__ __forceinline__ void g5_gradient_step(const d_sdf_shape* shape1,
                                                 const d_sdf_shape* shape2, v3 x0,
                                                 const d_opt_params* params, int niter,
                                                 bool sfd_intersection, float* out_dist,
                                                 v3* out_x) {
  const float amin = 1.0e-4f;
  const float rho = 0.5f;
  const float c = 0.1f;
  float dist = 1.0e10f;
  v3 x = x0;
  for (int iteration = 0; iteration < niter; ++iteration) {
    float alpha = 2.0f;
    v3 x2 = x;
    v3 x1 = v3_add(mul_mat3_vec(params->rel_mat, x2), params->rel_pos);
    v3 grad = g5_compute_grad(shape1, shape2, x1, x2, params, sfd_intersection);
    float dist0 = g5_clearance(shape1, shape2, x1, x, sfd_intersection);
    float grad_dot = v3_dot(grad, grad);
    if (grad_dot < 1.0e-12f) {
      *out_dist = dist0;
      *out_x = x;
      return;
    }
    float wolfe = -c * alpha * grad_dot;
    for (;;) {
      alpha *= rho;
      wolfe *= rho;
      x = v3_sub(x2, v3_scale(grad, alpha));
      x1 = v3_add(mul_mat3_vec(params->rel_mat, x), params->rel_pos);
      dist = g5_clearance(shape1, shape2, x1, x, sfd_intersection);
      if (alpha <= amin || (dist - dist0) <= wolfe) break;
    }
    if (dist > dist0) {
      *out_dist = dist0;
      *out_x = x2;
      return;
    }
  }
  *out_dist = dist;
  *out_x = x;
}

// 上游 `gradient_descent`：sdf_iterations 次普通步 + 1 次 intersection 步。
__device__ __forceinline__ void g5_gradient_descent(const d_sdf_shape* shape1,
                                                    const d_sdf_shape* shape2, v3 x0_initial,
                                                    v3 pos1, const float* rot1, v3 pos2,
                                                    const float* rot2, int sdf_iterations,
                                                    float* out_dist, v3* out_pos,
                                                    v3* out_normal) {
  d_opt_params params;
  relative_matrix(rot1, rot2, params.rel_mat);
  params.rel_pos = transpose_mul_mat3_vec(rot1, v3_sub(pos2, pos1));
  params.attr1 = shape1->size;
  params.attr2 = shape2->size;
  float dist = 0.0f;
  v3 x;
  g5_gradient_step(shape1, shape2, x0_initial, &params, sdf_iterations, false, &dist, &x);
  g5_gradient_step(shape1, shape2, x, &params, 1, true, &dist, &x);
  v3 x1 = v3_add(mul_mat3_vec(params.rel_mat, x), params.rel_pos);
  v3 grad1 = v3_make(0.0f, 0.0f, 0.0f);
  g5_sdf_grad(shape1, x1, &grad1);
  grad1 = v3_normalize(transpose_mul_mat3_vec(params.rel_mat, grad1));
  v3 grad2 = v3_make(0.0f, 0.0f, 0.0f);
  g5_forced_sdf_grad(shape2, x, &grad2);
  grad2 = v3_normalize(grad2);
  v3 n = v3_normalize(v3_sub(grad1, grad2));
  v3 pos = v3_add(mul_mat3_vec(rot2, x), pos2);
  n = mul_mat3_vec(rot2, n);
  *out_dist = dist;
  *out_pos = v3_sub(pos, v3_scale(n, dist / 2.0f));
  *out_normal = n;
}
// G10@P5 G5：SDF narrowphase 设备版，与 host `sdf/narrowphase.rs`（上游 `_sdf_narrowphase` 子集）对应。
// Adapted from frozen MJWarp collision_sdf.py, revision 71da24d. Apache-2.0.
// 设备结果直接对同配方上游 f32 比较（不依赖 host 侧 rel_mat 旧约定）。
#pragma once

struct d_sdf_pair {
  d_sdf_shape shape1;
  d_sdf_shape shape2;
  v3 pos1;
  float rot1[9];
  v3 pos2;
  float rot2[9];
  // 局部 AABB：(center.xyz, half_size.xyz)。
  float aabb1[6];
  float aabb2[6];
  int sdf_initpoints;
  int sdf_iterations;
};

struct d_sdf_contact {
  float dist;
  v3 pos;
  float frame[9];
};

// 上游 `_sdf_narrowphase` 的单 pair 子集：AABB 交集 + 每初始点一次梯度下降 → 接触候选。
// `capacity` 为调用方给出的接触容量（>= sdf_initpoints 时不会截断）；返回写入数量。
__device__ __forceinline__ int g5_sdf_pair_contacts(const d_sdf_pair* pair,
                                                    d_sdf_contact* out, int capacity) {
  const float identity[9] = {1.0f, 0.0f, 0.0f, 0.0f, 1.0f, 0.0f, 0.0f, 0.0f, 1.0f};
  v3 aabb1_center = v3_make(pair->aabb1[0], pair->aabb1[1], pair->aabb1[2]);
  v3 aabb1_half = v3_make(pair->aabb1[3], pair->aabb1[4], pair->aabb1[5]);
  v3 aabb1_min;
  v3 aabb1_max;
  g5_transform_aabb(aabb1_center, aabb1_half, v3_make(0.0f, 0.0f, 0.0f), identity,
                    &aabb1_min, &aabb1_max);

  float rel_mat[9];
  relative_matrix(pair->rot1, pair->rot2, rel_mat);
  v3 rel_pos = transpose_mul_mat3_vec(pair->rot1, v3_sub(pair->pos2, pair->pos1));
  v3 aabb2_center = v3_make(pair->aabb2[0], pair->aabb2[1], pair->aabb2[2]);
  v3 aabb2_half = v3_make(pair->aabb2[3], pair->aabb2[4], pair->aabb2[5]);
  v3 aabb2_min;
  v3 aabb2_max;
  g5_transform_aabb(aabb2_center, aabb2_half, rel_pos, rel_mat, &aabb2_min, &aabb2_max);

  v3 min_v = v3_make(fmaxf(aabb1_min.x, aabb2_min.x), fmaxf(aabb1_min.y, aabb2_min.y),
                     fmaxf(aabb1_min.z, aabb2_min.z));
  v3 max_v = v3_make(fminf(aabb1_max.x, aabb2_max.x), fminf(aabb1_max.y, aabb2_max.y),
                     fminf(aabb1_max.z, aabb2_max.z));
  if (min_v.x > max_v.x || min_v.y > max_v.y || min_v.z > max_v.z) return 0;

  int count = 0;
  int limit = pair->sdf_initpoints < capacity ? pair->sdf_initpoints : capacity;
  for (int index = 0; index < limit; ++index) {
    unsigned i = (unsigned)index;
    v3 x_g2 = v3_make(min_v.x + (max_v.x - min_v.x) * g5_halton(i, 2),
                      min_v.y + (max_v.y - min_v.y) * g5_halton(i, 3),
                      min_v.z + (max_v.z - min_v.z) * g5_halton(i, 5));
    v3 x = v3_add(mul_mat3_vec(pair->rot1, x_g2), pair->pos1);
    v3 x0_initial = transpose_mul_mat3_vec(pair->rot2, v3_sub(x, pair->pos2));
    float dist = 0.0f;
    v3 pos;
    v3 normal;
    g5_gradient_descent(&pair->shape1, &pair->shape2, x0_initial, pair->pos1, pair->rot1,
                        pair->pos2, pair->rot2, pair->sdf_iterations, &dist, &pos, &normal);
    out[count].dist = dist;
    out[count].pos = pos;
    make_frame(normal, out[count].frame);
    count += 1;
  }
  return count;
}
// G10@P5 G5：hfield prism CCD 设备版，与 host `hfield/{filter,driver}.rs` 逐式对应。
// Adapted from frozen MJWarp collision_convex.py (`_hfield_filter` / `ccd_hfield_kernel`),
// revision 71da24d. Apache-2.0.
// 世界→hfield 帧一律用 `common.cu::relative_matrix`（上游 `transpose(rot1)*rot2`），
// 不复制 host 测试 helper 的旧转置约定（详见 `src/physics/collision/relmat.rs`）。
#pragma once

// 上游 `MJ_MAXCONPAIR`（`mujoco.mjMAXCONPAIR`）。
#define G5_MJ_MAXCONPAIR 50
// 上游 `MJ_MAXVAL`（`mujoco.mjMAXVAL`）。
#define G5_MJ_MAXVAL 1.0e10f
// 上游 `MIN_DIST_TO_NEXT_CONTACT`。
#define G5_MIN_DIST_TO_NEXT_CONTACT 1.0e-3f
// 上游 `OverflowType.HFIELD`。
#define G5_OVERFLOW_HFIELD (1u << 5)

// hfield 模型视图（G1 上传口径：`hfield_size`/`hfield_nrow`/`hfield_ncol`/
// `hfield_adr`/`hfield_data`，全 f32）。
struct d_hfield_data {
  // (x, y, z_top, z_bottom)。
  float size[4];
  int nrow;
  int ncol;
  // `hfield_data` 起始下标。
  int adr;
  const float* data;
};

// 上游 `_hfield_filter` 的紧 AABB 边界（hfield 局部帧）。
struct d_hfield_bounds {
  float xmin;
  float xmax;
  float ymin;
  float ymax;
  float zmin;
  float zmax;
};

// 上游 `_hfield_filter`：返回 true 表示存在可碰撞棱柱（上游 `no_hf_collision=False`）。
// `geom2_in` 为世界系视图（与上游一致，在函数内变换到 hfield 帧）；
// `other_rbound` = `geom_rbound[g2]`，`margin` = 两侧 `geom_margin` 之和
// （模型数组汇总由宿主 launcher 预计算，与 host `hfield_filter` 入参一致）。
__device__ __forceinline__ bool g5_hfield_filter(const d_hfield_data& hfield,
                                                 const d_geom& geom2_in, int geomtype2,
                                                 float other_rbound, float margin, v3 hf_pos,
                                                 const float* hf_mat,
                                                 d_hfield_bounds* bounds) {
  const float* size1 = hfield.size;
  // 上游 `pos = mat1T @ (pos2 - pos1)`。
  v3 pos = transpose_mul_mat3_vec(hf_mat, v3_sub(geom2_in.pos, hf_pos));

  // box-sphere test: horizontal plane（`size[0]`、`size[1]` 为半宽）。
  for (int axis = 0; axis < 2; ++axis) {
    float p = axis == 0 ? pos.x : pos.y;
    if (size1[axis] < p - other_rbound - margin || -size1[axis] > p + other_rbound + margin) {
      return false;
    }
  }
  // box-sphere test: vertical direction（`size[2]` 顶、`size[3]` 底）。
  if (size1[2] < pos.z - other_rbound - margin) return false;
  if (-size1[3] > pos.z + other_rbound + margin) return false;

  // 支撑紧 AABB：geom2 在 hfield 帧内、margin=0（上游同）。
  d_geom geom2 = geom2_in;
  geom2.pos = pos;
  // 上游 `geom2.rot = mat1T @ mat2`。
  relative_matrix(hf_mat, geom2_in.rot, geom2.rot);
  geom2.margin = 0.0f;
  geom2.index = -1;

  float xmax = support(geom2, geomtype2, v3_make(1.0f, 0.0f, 0.0f)).point.x;
  float xmin = support(geom2, geomtype2, v3_make(-1.0f, 0.0f, 0.0f)).point.x;
  float ymax = support(geom2, geomtype2, v3_make(0.0f, 1.0f, 0.0f)).point.y;
  float ymin = support(geom2, geomtype2, v3_make(0.0f, -1.0f, 0.0f)).point.y;
  float zmax = support(geom2, geomtype2, v3_make(0.0f, 0.0f, 1.0f)).point.z;
  float zmin = support(geom2, geomtype2, v3_make(0.0f, 0.0f, -1.0f)).point.z;

  // box-box test。
  if (xmin - margin > size1[0] || xmax + margin < -size1[0] ||
      ymin - margin > size1[1] || ymax + margin < -size1[1] ||
      zmin - margin > size1[2] || zmax + margin < -size1[3]) {
    return false;
  }
  bounds->xmin = xmin;
  bounds->xmax = xmax;
  bounds->ymin = ymin;
  bounds->ymax = ymax;
  bounds->zmin = zmin;
  bounds->zmax = zmax;
  return true;
}

// 一个 hfield 接触（世界系，与上游 `write_contact` 入参一致）。
struct d_hfield_contact {
  float dist;
  v3 pos;
  // 行优先 3×3 frame（由 `make_frame(normal)` 生成）。
  float frame[9];
};

// 单对 hfield 结果；`overflow` 汇总本对产生的溢出位（HFIELD + 逐棱柱 ccd 溢出）。
struct d_hfield_result {
  // 实际写入的接触数（0..4）。
  int ncontact;
  // 上游 `count`：通过筛选且进入 `ccd()` 并命中接触的棱柱数（可 >= 4）。
  int prism_count;
  unsigned overflow;
  d_hfield_contact contact[4];
};

// 上游 `write_contact` 的接触装配子集（frame 由法线生成；nacon 写入归第五批）。
__device__ __forceinline__ void g5_write_hfield_contact(d_hfield_contact* contact, float dist,
                                                        v3 pos, v3 normal) {
  contact->dist = dist;
  contact->pos = pos;
  make_frame(normal, contact->frame);
}

// 上游 `ccd_hfield_kernel` 的单对主体：hfield 子网格棱柱扫描 + `ccd()` + 0..3 接触选择。
// `pt` 为调用方按 `epa_iterations` 提供的 EPA polytope 缓冲（上游按 ccdid 复用同一缓冲）。
__device__ __forceinline__ void g5_hfield_pair_contacts(
    const d_hfield_data& hfield, v3 hf_pos, const float* hf_mat, const d_geom& geom2_in,
    int geomtype2, float other_rbound, float margin, float gap, d_ccd_options options,
    d_polytope* pt, d_hfield_result* out) {
  out->ncontact = 0;
  out->prism_count = 0;
  out->overflow = 0u;
  for (int i = 0; i < 4; ++i) {
    out->contact[i].dist = 0.0f;
    out->contact[i].pos = v3_make(0.0f, 0.0f, 0.0f);
    for (int k = 0; k < 9; ++k) out->contact[i].frame[k] = 0.0f;
  }

  d_hfield_bounds bounds;
  if (!g5_hfield_filter(hfield, geom2_in, geomtype2, other_rbound, margin, hf_pos, hf_mat,
                        &bounds)) {
    return;
  }

  // 上游：geom2 变换到 hfield 帧；geom1 恒为单位位姿。
  d_geom geom2 = geom2_in;
  geom2.pos = transpose_mul_mat3_vec(hf_mat, v3_sub(geom2_in.pos, hf_pos));
  relative_matrix(hf_mat, geom2_in.rot, geom2.rot);

  d_geom geom1;
  geom1.pos = v3_make(0.0f, 0.0f, 0.0f);
  const float identity[9] = {1.0f, 0.0f, 0.0f, 0.0f, 1.0f, 0.0f, 0.0f, 0.0f, 1.0f};
  for (int k = 0; k < 9; ++k) geom1.rot[k] = identity[k];
  geom1.size = v3_make(0.0f, 0.0f, 0.0f);
  geom1.margin = 0.0f;
  for (int k = 0; k < 18; ++k) geom1.polyvert[k] = 0.0f;
  geom1.vertadr = -1;
  geom1.vertnum = -1;
  geom1.graphadr = -1;
  geom1.index = -1;
  geom1.dataid = -1;
  geom1.mesh_polyadr = -1;
  geom1.mesh.vert = nullptr;
  geom1.mesh.graph = nullptr;
  g5_mesh_clear_poly(&geom1.mesh);

  const float size[4] = {hfield.size[0], hfield.size[1], hfield.size[2], hfield.size[3]};
  int nrow = hfield.nrow;
  int ncol = hfield.ncol;

  // 上游 hfield 子网格。
  float x_scale = 0.5f * (float)(ncol - 1) / size[0];
  float y_scale = 0.5f * (float)(nrow - 1) / size[1];
  int cmin = (int)floorf((bounds.xmin + size[0]) * x_scale);
  if (cmin < 0) cmin = 0;
  int cmax = (int)ceilf((bounds.xmax + size[0]) * x_scale);
  if (cmax > ncol - 1) cmax = ncol - 1;
  int rmin = (int)floorf((bounds.ymin + size[1]) * y_scale);
  if (rmin < 0) rmin = 0;
  int rmax = (int)ceilf((bounds.ymax + size[1]) * y_scale);
  if (rmax > nrow - 1) rmax = nrow - 1;

  float dx = (2.0f * size[0]) / (float)(ncol - 1);
  float dy = (2.0f * size[1]) / (float)(nrow - 1);
  const int dr[2] = {1, 0};

  // prism 六顶点（行优先 6×3）：0..2 底三角（z=-size[3]），3..5 顶三角。
  float prism[18] = {0.0f};
  prism[2] = -size[3];
  prism[5] = -size[3];
  prism[8] = -size[3];

  int adr = hfield.adr;

  float contact_dist[G5_MJ_MAXCONPAIR];
  v3 contact_pos[G5_MJ_MAXCONPAIR];
  v3 contact_normal[G5_MJ_MAXCONPAIR];
  float min_dist = G5_MJ_MAXVAL;
  v3 min_normal = v3_make(G5_MJ_MAXVAL, G5_MJ_MAXVAL, G5_MJ_MAXVAL);
  v3 min_pos = v3_make(G5_MJ_MAXVAL, G5_MJ_MAXVAL, G5_MJ_MAXVAL);
  int min_id = -1;

  // 上游在本阶段把 geom2.margin 设为两侧 margin 之和（geom1 margin 恒 0）。
  geom2.margin = margin;

  int count = 0;
  int noverflow = 0;
  for (int r = rmin; r < rmax; ++r) {
    if (noverflow > 0) break;

    // 预置本行前两个顶点（行首列）。
    for (int init_i = 0; init_i < 2; ++init_i) {
      float x = dx * (float)cmin - size[0];
      float y = dy * (float)(r + dr[init_i]) - size[1];
      float z = hfield.data[adr + (r + dr[init_i]) * ncol + cmin] * size[2] + margin;

      for (int k = 0; k < 3; ++k) prism[k] = prism[3 + k];
      for (int k = 0; k < 3; ++k) prism[3 + k] = prism[6 + k];
      for (int k = 0; k < 3; ++k) prism[9 + k] = prism[12 + k];
      for (int k = 0; k < 3; ++k) prism[12 + k] = prism[15 + k];
      prism[6] = x;
      prism[15] = x;
      prism[7] = y;
      prism[16] = y;
      prism[17] = z;
    }

    for (int c = cmin + 1; c <= cmax; ++c) {
      if (noverflow > 0) break;

      // 每格两个三角棱柱。
      for (int i = 0; i < 2; ++i) {
        if (count >= G5_MJ_MAXCONPAIR) {
          noverflow = 1;
          break;
        }

        // 追加本列顶点。
        float x = dx * (float)c - size[0];
        float y = dy * (float)(r + dr[i]) - size[1];
        float z = hfield.data[adr + (r + dr[i]) * ncol + c] * size[2] + margin;

        for (int k = 0; k < 3; ++k) prism[k] = prism[3 + k];
        for (int k = 0; k < 3; ++k) prism[3 + k] = prism[6 + k];
        for (int k = 0; k < 3; ++k) prism[9 + k] = prism[12 + k];
        for (int k = 0; k < 3; ++k) prism[12 + k] = prism[15 + k];
        prism[6] = x;
        prism[15] = x;
        prism[7] = y;
        prism[16] = y;
        prism[17] = z;

        // prism 高度测试（顶三角 z 均低于紧 AABB 下界）。
        if (prism[11] < bounds.zmin && prism[14] < bounds.zmin && prism[17] < bounds.zmin) {
          continue;
        }

        for (int k = 0; k < 18; ++k) geom1.polyvert[k] = prism[k];

        // 上游 `x1 = geom1.pos + geom1.rot @ (sum * 1/6)`。
        v3 sum = v3_make(0.0f, 0.0f, 0.0f);
        for (int k = 0; k < 6; ++k) {
          sum = v3_add(sum, v3_make(prism[3 * k], prism[3 * k + 1], prism[3 * k + 2]));
        }
        v3 center = v3_add(geom1.pos, mul_mat3_vec(geom1.rot, v3_scale(sum, 1.0f / 6.0f)));

        d_ccd_result result;
        g5_ccd(geom1, geom2, G5_HFIELD, geomtype2, center, geom2.pos, options, pt, &result);
        // 上游把 `overflow_out` 透传给 ccd（只可能有 `EPA_HORIZON`），由调用方汇总；
        // `CCD` 位只在 `nccd >= naccdmax` 的真实容量截断时置（见 contact.cu）。
        out->overflow |= result.overflow;

        // 分离棱柱跳过（ccd 对分离返回正距离）。
        if (result.ncontact == 0 || result.dist >= margin + gap) continue;

        // 上游：`pos_local = 0.5 * (w1 + w2)`。
        v3 pos_local = v3_scale(v3_add(result.x1, result.x2), 0.5f);
        v3 pos = v3_add(mul_mat3_vec(hf_mat, pos_local), hf_pos);

        float frame_local[9];
        make_frame(v3_sub(result.x1, result.x2), frame_local);
        v3 normal_local = v3_make(frame_local[0], frame_local[1], frame_local[2]);
        v3 normal = mul_mat3_vec(hf_mat, normal_local);

        contact_dist[count] = result.dist;
        contact_pos[count] = pos;
        contact_normal[count] = normal;
        if (result.dist < min_dist) {
          min_dist = result.dist;
          min_normal = normal;
          min_pos = pos;
          min_id = count;
        }
        count += 1;
      }
    }
  }

  // 上游：每对只报一次 HFIELD 溢出（`MJ_MAXCONPAIR`）。
  if (noverflow > 0) out->overflow |= G5_OVERFLOW_HFIELD;
  out->prism_count = count;
  if (count == 0) return;

  // contact 0：最小距离接触。
  g5_write_hfield_contact(&out->contact[0], min_dist, min_pos, min_normal);
  out->ncontact = 1;

  // contact 1：离最小距离接触点最远的接触。
  int id1 = -1;
  float dist1 = -G5_MJ_MAXVAL;
  for (int i = 0; i < count; ++i) {
    if (i == min_id) continue;
    float dist = v3_length(v3_sub(contact_pos[i], min_pos));
    if (dist > dist1) {
      id1 = i;
      dist1 = dist;
    }
  }
  if (id1 == -1 || (dist1 > 0.0f && dist1 < G5_MIN_DIST_TO_NEXT_CONTACT)) return;
  v3 pos1 = contact_pos[id1];
  g5_write_hfield_contact(&out->contact[1], contact_dist[id1], pos1, contact_normal[id1]);
  out->ncontact = 2;

  // contact 2：离 min_pos→pos1 直线最远的接触。
  v3 dist_min1 = v3_cross(min_normal, v3_sub(min_pos, pos1));
  int id2 = -1;
  float dist_12 = -G5_MJ_MAXVAL;
  for (int i = 0; i < count; ++i) {
    if (i == min_id || i == id1) continue;
    float dist = fabsf(v3_dot(v3_sub(contact_pos[i], min_pos), dist_min1));
    if (dist > dist_12) {
      id2 = i;
      dist_12 = dist;
    }
  }
  if (id2 == -1 || (dist_12 > 0.0f && dist_12 < G5_MIN_DIST_TO_NEXT_CONTACT)) return;
  v3 pos2 = contact_pos[id2];
  g5_write_hfield_contact(&out->contact[2], contact_dist[id2], pos2, contact_normal[id2]);
  out->ncontact = 3;

  // contact 3：离另一条三角边最远的接触。
  v3 vec_min2 = v3_cross(min_normal, v3_sub(min_pos, pos2));
  v3 vec_12 = v3_cross(min_normal, v3_sub(pos1, pos2));
  int id3 = -1;
  float dist3 = -G5_MJ_MAXVAL;
  for (int i = 0; i < count; ++i) {
    if (i == min_id || i == id1 || i == id2) continue;
    v3 point = contact_pos[i];
    float dist = fabsf(v3_dot(v3_sub(point, min_pos), vec_min2)) +
                 fabsf(v3_dot(v3_sub(pos1, point), vec_12));
    if (dist > dist3) {
      id3 = i;
      dist3 = dist;
    }
  }
  if (id3 == -1 || (dist3 > 0.0f && dist3 < G5_MIN_DIST_TO_NEXT_CONTACT)) return;
  g5_write_hfield_contact(&out->contact[3], contact_dist[id3], contact_pos[id3],
                          contact_normal[id3]);
  out->ncontact = 4;
}
// G10@P5 G5：multicontact 的 BOX/CYLINDER 几何助手设备版，与 host
// `gjk/multicontact/{feature,normals}.rs` 逐式对应。
// Adapted from frozen MJWarp collision_gjk.py L1533-1587（`_feature_dim` / `_aligned_faces` /
// `_aligned_face_edge`）、L1739-1922（`_box_normals2` / `_box_normals` / `_box_edge_normals` /
// `_box_face`）、L1953-2023（`_cylinder_normals` / `_cylinder_edge_normals` / `_cylinder_face`）、
// L2199-2211（`_set_edge`）, revision 71da24d. Apache-2.0.
//
// 本片只覆盖 BOX/CYLINDER。mesh 多接触不实现：上游 `_mesh_normals`（L1621）、
// `_mesh_edge_normals`（L1691）、`_mesh_face`（L1927）及其助手 `_intersect1`/`_intersect2`
// （L1592/L1606）整体省略；mesh 类型在 `multicontact.cu` 直接回退单接触。
//
// 数值纪律：全程 f32、与上游相同操作数顺序、不做代数化简（照写 `a*b+c`，收缩交给
// `--fmad=true`）。常量按上游 f64 计算值落到 f32：容差宏在行尾给出位型，`_CYLINDER_*_16`
// 两个表的十进制均为 f32 最短回读字面量（已逐项回读校验 == host `gjk/mod.rs` 的 from_bits）。
#pragma once

// 上游 L46-47：`FACE_TOL = cos(0.0889)`、`EDGE_TOL = sin(0.0889)`。
#define G5_MC_FACE_TOL 0.996051f      // 0x3F7EFD33
#define G5_MC_EDGE_TOL 0.0887829438f  // 0x3DB5D3D5
// 上游 L33-34：`MINVAL` 用于 `_aligned_face_edge` 的朝向门；`MINVAL2` 在 L1533-2451
// 的调用链里未被引用，仅为常量完整性保留。
#define G5_MC_MINVAL 1.0e-15f
#define G5_MC_MINVAL2 1.0e-30f

// 上游 `float(bool)`：整数掩码非零的真值。
__device__ __forceinline__ bool g5_mc_bit(int value, int mask) {
    return (value & mask) != 0;
}

// 取行优先 mat33 的第 col 列（上游 `mat[:, col]`）。
// 上游 `mat33` 的 `mat[:, j]` 是列向量：行优先存储下为 (m[j], m[3+j], m[6+j])。
__device__ __forceinline__ v3 g5_mc_mat_col(const float* mat, int col) {
    return v3_make(mat[col], mat[3 + col], mat[6 + col]);
}

// 上游 `mat @ vec3 + pos`（行优先 matvec 后再平移）。
__device__ __forceinline__ v3 g5_mc_rot_pos(const float* mat, v3 v, v3 pos) {
    return v3_add(mul_mat3_vec(mat, v), pos);
}

// 上游 `_feature_dim`（L1533）：返回 feature 维度（1/2/3），必要时重排 index/vertex 的第 2 行。
// `face` 为 EPA 面解包出的三个 polytope 顶点号；`offset` 0 = geom1，1 = geom2。
__device__ __forceinline__ int g5_mc_feature_dim(const int* face, const int* vert_index,
                                                 const v3* vert, int offset, int* feature_index,
                                                 v3* feature_vertex) {
    int v1i = vert_index[2 * face[0] + offset];
    int v2i = vert_index[2 * face[1] + offset];
    int v3i = vert_index[2 * face[2] + offset];

    feature_index[0] = v1i;
    feature_index[1] = v2i;
    feature_index[2] = v3i;
    feature_vertex[0] = vert[2 * face[0] + offset];
    feature_vertex[1] = vert[2 * face[1] + offset];
    feature_vertex[2] = vert[2 * face[2] + offset];

    if (v1i != v2i) {
        return (v3i == v1i || v3i == v2i) ? 2 : 3;
    }

    feature_index[1] = v3i;
    feature_vertex[1] = vert[2 * face[2] + offset];

    return (v1i != v3i) ? 2 : 1;
}

// 上游 `_aligned_faces`（L1559）：找一对相互朝向（dot < -FACE_TOL）的法线，返回 1 表示找到。
// 命中时 `res = {i, j}`（两个法线数组的下标）；未命中保持 {0, 0}。
__device__ __forceinline__ int g5_mc_aligned_faces(const v3* vert1, int len1, const v3* vert2,
                                                   int len2, int* res) {
    res[0] = 0;
    res[1] = 0;
    for (int i = 0; i < len1; ++i) {
        for (int j = 0; j < len2; ++j) {
            if (v3_dot(vert1[i], vert2[j]) < -G5_MC_FACE_TOL) {
                res[0] = i;
                res[1] = j;
                return 1;
            }
        }
    }
    return 0;
}

// 上游 `_aligned_face_edge`（L1573）：找一条与面垂直（|dot| < EDGE_TOL）的边。
// 命中时 `res = {边下标, 面下标}`。
__device__ __forceinline__ int g5_mc_aligned_face_edge(const v3* edge, int nedge, const v3* face,
                                                       int nface, v3 dir, int* res) {
    res[0] = 0;
    res[1] = 0;
    for (int i = 0; i < nface; ++i) {
        // 忽略背向碰撞方向的面（负点积）。
        if (v3_dot(face[i], dir) <= G5_MC_MINVAL) continue;

        for (int j = 0; j < nedge; ++j) {
            if (fabsf(v3_dot(edge[j], face[i])) < G5_MC_EDGE_TOL) {
                res[0] = j;
                res[1] = i;
                return 1;
            }
        }
    }
    return 0;
}

// 上游 `_set_edge`（L2199）：用 EPA 顶点与 endvert 拼出 2 点“面”。
__device__ __forceinline__ int g5_mc_set_edge(const v3* vert1, const v3* vert2, int start, int end,
                                              int offset, v3* face_out) {
    face_out[0] = vert1[2 * start + offset];
    face_out[1] = vert2[end];
    return 2;
}

// 上游 `_box_normals2`（L1739）：由碰撞法线反推最近的 box 面法线。
__device__ __forceinline__ int g5_mc_box_normals2(const float* mat, v3 n, v3* normal_out,
                                                  int* index_out) {
    // list of box face normals
    const v3 face_normals[6] = {v3_make(1.0f, 0.0f, 0.0f),  v3_make(-1.0f, 0.0f, 0.0f),
                                v3_make(0.0f, 1.0f, 0.0f),  v3_make(0.0f, -1.0f, 0.0f),
                                v3_make(0.0f, 0.0f, 1.0f),  v3_make(0.0f, 0.0f, -1.0f)};

    // get local coordinates of the normal（上游逐项写 transpose(mat) @ n）
    v3 local_n = v3_normalize(transpose_mul_mat3_vec(mat, n));

    // determine if there is a side close to the normal
    for (int i = 0; i < 6; ++i) {
        if (v3_dot(local_n, face_normals[i]) > G5_MC_FACE_TOL) {
            normal_out[0] = mul_mat3_vec(mat, face_normals[i]);
            index_out[0] = i;
            return 1;
        }
    }

    return 0;
}

// 上游 `_box_normals`（L1772）：由最多三个顶点恢复 box 面法线（含 index 语义）。
__device__ __forceinline__ int g5_mc_box_normals(int feature_dim, const int* feature_index,
                                                 const float* mat, v3 dir, v3* normal_out,
                                                 int* index_out) {
    int v1 = feature_index[0];
    int v2 = feature_index[1];
    int v3 = feature_index[2];

    if (feature_dim == 3) {
        int c = 0;
        float x = (float)(g5_mc_bit(v1, 1) && g5_mc_bit(v2, 1) && g5_mc_bit(v3, 1)) -
                  (float)(!g5_mc_bit(v1, 1) && !g5_mc_bit(v2, 1) && !g5_mc_bit(v3, 1));
        float y = (float)(g5_mc_bit(v1, 2) && g5_mc_bit(v2, 2) && g5_mc_bit(v3, 2)) -
                  (float)(!g5_mc_bit(v1, 2) && !g5_mc_bit(v2, 2) && !g5_mc_bit(v3, 2));
        float z = (float)(g5_mc_bit(v1, 4) && g5_mc_bit(v2, 4) && g5_mc_bit(v3, 4)) -
                  (float)(!g5_mc_bit(v1, 4) && !g5_mc_bit(v2, 4) && !g5_mc_bit(v3, 4));
        normal_out[0] = mul_mat3_vec(mat, v3_make(x, y, z));
        float sgn = x + y + z;
        if (x != 0.0f) {
            index_out[c] = 0;
            c += 1;
        }
        if (y != 0.0f) {
            index_out[c] = 2;
            c += 1;
        }
        if (z != 0.0f) {
            index_out[c] = 4;
            c += 1;
        }
        if (sgn == -1.0f) {
            index_out[0] = index_out[0] + 1;
        }
        if (c == 1) {
            return 1;
        }
        return g5_mc_box_normals2(mat, dir, normal_out, index_out);
    }
    if (feature_dim == 2) {
        int c = 0;
        float x = (float)(g5_mc_bit(v1, 1) && g5_mc_bit(v2, 1)) -
                  (float)(!g5_mc_bit(v1, 1) && !g5_mc_bit(v2, 1));
        float y = (float)(g5_mc_bit(v1, 2) && g5_mc_bit(v2, 2)) -
                  (float)(!g5_mc_bit(v1, 2) && !g5_mc_bit(v2, 2));
        float z = (float)(g5_mc_bit(v1, 4) && g5_mc_bit(v2, 4)) -
                  (float)(!g5_mc_bit(v1, 4) && !g5_mc_bit(v2, 4));
        if (x != 0.0f) {
            normal_out[c] = mul_mat3_vec(mat, v3_make(x, 0.0f, 0.0f));
            index_out[c] = (x > 0.0f) ? 0 : 1;
            c += 1;
        }
        if (y != 0.0f) {
            normal_out[c] = mul_mat3_vec(mat, v3_make(0.0f, y, 0.0f));
            index_out[c] = (y > 0.0f) ? 2 : 3;
            c += 1;
        }
        if (z != 0.0f) {
            normal_out[c] = mul_mat3_vec(mat, v3_make(0.0f, 0.0f, z));
            index_out[c] = (z > 0.0f) ? 4 : 5;
            c += 1;
        }
        // c is 1 if edge is diagonal of a box face
        // c is 2 if edge is an external edge of box
        if (c == 1 || c == 2) {
            return c;
        }
        return g5_mc_box_normals2(mat, dir, normal_out, index_out);
    }

    if (feature_dim == 1) {
        float x = g5_mc_bit(v1, 1) ? 1.0f : -1.0f;
        float y = g5_mc_bit(v1, 2) ? 1.0f : -1.0f;
        float z = g5_mc_bit(v1, 4) ? 1.0f : -1.0f;
        normal_out[0] = mul_mat3_vec(mat, v3_make(x, 0.0f, 0.0f));
        normal_out[1] = mul_mat3_vec(mat, v3_make(0.0f, y, 0.0f));
        normal_out[2] = mul_mat3_vec(mat, v3_make(0.0f, 0.0f, z));
        index_out[0] = (x > 0.0f) ? 0 : 1;
        index_out[1] = (y > 0.0f) ? 2 : 3;
        index_out[2] = (z > 0.0f) ? 4 : 5;
        return 3;
    }
    return 0;
}

// 上游 `_box_edge_normals`（L1846）：box 的边-面碰撞候选方向（dim==1 时 3 条相邻边）。
__device__ __forceinline__ int g5_mc_box_edge_normals(int dim, const float* mat, v3 pos, v3 size,
                                                      v3 v1, v3 v2, int v1i, v3* normal_out,
                                                      v3* endvert_out) {
    if (dim == 2) {
        endvert_out[0] = v2;
        normal_out[0] = v3_normalize(v3_sub(v2, v1));
        return 1;
    }

    // return 3 adjacent vertices
    if (dim == 1) {
        float x = g5_mc_bit(v1i, 1) ? size.x : -size.x;
        float y = g5_mc_bit(v1i, 2) ? size.y : -size.y;
        float z = g5_mc_bit(v1i, 4) ? size.z : -size.z;

        endvert_out[0] = g5_mc_rot_pos(mat, v3_make(-x, y, z), pos);
        normal_out[0] = v3_normalize(v3_sub(endvert_out[0], v1));

        endvert_out[1] = g5_mc_rot_pos(mat, v3_make(x, -y, z), pos);
        normal_out[1] = v3_normalize(v3_sub(endvert_out[1], v1));

        endvert_out[2] = g5_mc_rot_pos(mat, v3_make(x, y, -z), pos);
        normal_out[2] = v3_normalize(v3_sub(endvert_out[2], v1));
        return 3;
    }
    return 0;
}

// 上游 `_box_face`（L1884）：按面索引恢复 box 的 4 个世界顶点。
__device__ __forceinline__ int g5_mc_box_face(const float* mat, v3 pos, v3 size, int idx,
                                              v3* face_out) {
    // compute global coordinates of the box face and face normal
    float sx = size.x;
    float sy = size.y;
    float sz = size.z;
    if (idx == 0) {  // right
        face_out[0] = g5_mc_rot_pos(mat, v3_make(sx, sy, sz), pos);
        face_out[1] = g5_mc_rot_pos(mat, v3_make(sx, sy, -sz), pos);
        face_out[2] = g5_mc_rot_pos(mat, v3_make(sx, -sy, -sz), pos);
        face_out[3] = g5_mc_rot_pos(mat, v3_make(sx, -sy, sz), pos);
        return 4;
    }
    if (idx == 1) {  // left
        face_out[0] = g5_mc_rot_pos(mat, v3_make(-sx, sy, -sz), pos);
        face_out[1] = g5_mc_rot_pos(mat, v3_make(-sx, sy, sz), pos);
        face_out[2] = g5_mc_rot_pos(mat, v3_make(-sx, -sy, sz), pos);
        face_out[3] = g5_mc_rot_pos(mat, v3_make(-sx, -sy, -sz), pos);
        return 4;
    }
    if (idx == 2) {  // top
        face_out[0] = g5_mc_rot_pos(mat, v3_make(-sx, sy, -sz), pos);
        face_out[1] = g5_mc_rot_pos(mat, v3_make(sx, sy, -sz), pos);
        face_out[2] = g5_mc_rot_pos(mat, v3_make(sx, sy, sz), pos);
        face_out[3] = g5_mc_rot_pos(mat, v3_make(-sx, sy, sz), pos);
        return 4;
    }
    if (idx == 3) {  // bottom
        face_out[0] = g5_mc_rot_pos(mat, v3_make(-sx, -sy, sz), pos);
        face_out[1] = g5_mc_rot_pos(mat, v3_make(sx, -sy, sz), pos);
        face_out[2] = g5_mc_rot_pos(mat, v3_make(sx, -sy, -sz), pos);
        face_out[3] = g5_mc_rot_pos(mat, v3_make(-sx, -sy, -sz), pos);
        return 4;
    }
    if (idx == 4) {  // front
        face_out[0] = g5_mc_rot_pos(mat, v3_make(-sx, sy, sz), pos);
        face_out[1] = g5_mc_rot_pos(mat, v3_make(sx, sy, sz), pos);
        face_out[2] = g5_mc_rot_pos(mat, v3_make(sx, -sy, sz), pos);
        face_out[3] = g5_mc_rot_pos(mat, v3_make(-sx, -sy, sz), pos);
        return 4;
    }
    if (idx == 5) {  // back
        face_out[0] = g5_mc_rot_pos(mat, v3_make(sx, sy, -sz), pos);
        face_out[1] = g5_mc_rot_pos(mat, v3_make(-sx, sy, -sz), pos);
        face_out[2] = g5_mc_rot_pos(mat, v3_make(-sx, -sy, -sz), pos);
        face_out[3] = g5_mc_rot_pos(mat, v3_make(sx, -sy, -sz), pos);
        return 4;
    }
    return 0;
}

// 上游 `_cylinder_normals`（L1953）：由顶点索引反推圆柱端面法线（旋转矩阵第 3 列）。
__device__ __forceinline__ int g5_mc_cylinder_normals(int feature_dim, const int* feature_index,
                                                      const float* mat, v3* normal_out,
                                                      int* index_out) {
    if (feature_dim == 1) {
        float sgn = (feature_index[0] != 0) ? -1.0f : 1.0f;
        v3 col = g5_mc_mat_col(mat, 2);
        normal_out[0] = v3_make(col.x * sgn, col.y * sgn, col.z * sgn);
        index_out[0] = feature_index[0];
        return 1;
    }
    return 0;
}

// 上游 `_cylinder_edge_normals`（L1972）：由碰撞点恢复圆柱边方向与端点。
__device__ __forceinline__ int g5_mc_cylinder_edge_normals(int dim, const float* mat, v3 size, v3 v,
                                                           int v1i, v3* normal_out,
                                                           v3* endvert_out) {
    if (dim == 1 || dim == 2) {
        float sgn = (v1i != 0) ? 1.0f : -1.0f;
        v3 col = g5_mc_mat_col(mat, 2);
        v3 res = v3_make(col.x * sgn, col.y * sgn, col.z * sgn);
        normal_out[0] = res;
        endvert_out[0] = v3_add(v, v3_scale(res, 2.0f * size.y));
        return 1;
    }
    return 0;
}

// 上游 L59-61：`_CYLINDER_COS_16 = cos(i*pi/8)`、`_CYLINDER_SIN_16 = sin(i*pi/8)`（16 边形
// 端面近似）。十进制为 f32 最短回读字面量，逐项校验 == host `gjk/mod.rs` 的 from_bits 位型；
// 近零项 0x248D3132 / 0x250D3132 / 0xA553C9CA 即 f64 `math.cos(pi/2)` 等的 f32 舍入。
static __device__ const float g5_mc_cylinder_cos16[16] = {
    1.0f,           0.9238795f,     0.707106769f,   0.382683426f,  6.123234e-17f,
    -0.382683426f,  -0.707106769f,  -0.9238795f,    -1.0f,         -0.9238795f,
    -0.707106769f,  -0.382683426f,  -1.83697015e-16f, 0.382683426f, 0.707106769f,
    0.9238795f};

static __device__ const float g5_mc_cylinder_sin16[16] = {
    0.0f,           0.382683426f,   0.707106769f,   0.9238795f,    1.0f,
    0.9238795f,     0.707106769f,   0.382683426f,   1.22464685e-16f, -0.382683426f,
    -0.707106769f,  -0.9238795f,    -1.0f,          -0.9238795f,   -0.707106769f,
    -0.382683426f};

// 上游 `_cylinder_face`（L1994）：把圆柱端面近似成 16 边形并返回 16 个世界顶点。
__device__ __forceinline__ int g5_mc_cylinder_face(const float* mat, v3 pos, v3 size, int idx,
                                                   v3* face_out) {
    float sgn = (idx != 0) ? -1.0f : 1.0f;
    v3 center = v3_add(v3_scale(g5_mc_mat_col(mat, 2), sgn * size.y), pos);
    v3 col0 = v3_scale(g5_mc_mat_col(mat, 0), size.x);
    v3 col1 = v3_scale(g5_mc_mat_col(mat, 1), size.x * sgn);
    for (int i = 0; i < 16; ++i) {
        // 上游逐行写 `col0*cos[i] - col1*sin[i] + center`。
        face_out[i] = v3_add(v3_sub(v3_scale(col0, g5_mc_cylinder_cos16[i]),
                                    v3_scale(col1, g5_mc_cylinder_sin16[i])),
                             center);
    }
    return 16;
}

// ===== M2：mesh polygon 视图（convex-vs-mesh multicontact）=====
// 上游 `multicontact()`（collision_gjk.py L2245-2262）在 mesh 参与时取 mesh 侧的
// `polynormal/polyvertadr/polyvertnum/polyvert/polymapadr/polymapnum/polymap` 数组与
// `vert/vertadr` 视图。本实现的 9 个数组来自 M0a sidecar（形状语义与修正后的
// `validate_mesh_poly` 一致：polynum/polyadr 逐 mesh、polyvert* 逐 polygon、polymap* 逐 vertex），
// 逐 geom 的 `vertadr` 与 `mesh_polyadr` 由 `product.cu::build_geom` 按 `dataid` 解析进 `d_geom`。
struct g5_mc_mesh_view {
    const float* vert;
    const float* polynormal;
    const int* polyvertadr;
    const int* polyvertnum;
    const int* polyvert;
    const int* polymapadr;
    const int* polymapnum;
    const int* polymap;
    bool has_poly;
};

// scratch 容量上界（与 `contact.cu` 的 `n1/n2/endvert[10]`、`face1/face2[20]` 一致）：
// 超出时对应 mesh 助手返回 0，由 `g5_multicontact` 回退单接触（未覆盖边界，见证据）。
// A1（2026-10-31）：npolygonmax 16 -> 20（对应上游 npolygonmax=20）；degree 仍 10（U7 未覆盖，A2）。
#define G5_MC_MESH_DEG_MAX 10
#define G5_MC_MESH_FACE_MAX 20

__device__ __forceinline__ g5_mc_mesh_view g5_mc_mesh_view_of(const d_geom& mesh_geom) {
    g5_mc_mesh_view view;
    view.vert = mesh_geom.mesh.vert;
    view.polynormal = mesh_geom.mesh.polynormal;
    view.polyvertadr = mesh_geom.mesh.polyvertadr;
    view.polyvertnum = mesh_geom.mesh.polyvertnum;
    view.polyvert = mesh_geom.mesh.polyvert;
    view.polymapadr = mesh_geom.mesh.polymapadr;
    view.polymapnum = mesh_geom.mesh.polymapnum;
    view.polymap = mesh_geom.mesh.polymap;
    view.has_poly = (mesh_geom.mesh.polynum != nullptr) && (mesh_geom.mesh.polyadr != nullptr) &&
                    (mesh_geom.mesh.polynormal != nullptr) &&
                    (mesh_geom.mesh.polyvertadr != nullptr) &&
                    (mesh_geom.mesh.polyvertnum != nullptr) &&
                    (mesh_geom.mesh.polyvert != nullptr) &&
                    (mesh_geom.mesh.polymapadr != nullptr) &&
                    (mesh_geom.mesh.polymapnum != nullptr) &&
                    (mesh_geom.mesh.polymap != nullptr);
    return view;
}

// sidecar `mesh_polynormal` 的第 `row` 行（vec3 展平）→ v3。
__device__ __forceinline__ v3 g5_mc_mesh_poly_normal(const g5_mc_mesh_view& mesh, int row) {
    return g5_mesh_vertex(mesh.polynormal, row);
}
// G10@P5 M2：convex-vs-mesh multicontact 的 mesh 几何助手设备版。
// Adapted from frozen MJWarp collision_gjk.py `_intersect1`（L1591-1602）、`_intersect2`
// （L1605-1616）、`_mesh_normals`（L1620-1686）、`_mesh_edge_normals`（L1690-1734）、
// `_mesh_face`（L1926-1948），revision 71da24d. Apache-2.0.
//
// 数值纪律：全程 f32、与上游相同操作数顺序、不做代数化简（照写 `a*b+c`，收缩交给
// `--fmad=true`）；矩阵乘 `mat @ v` 用行优先 `mul_mat3_vec`（与上游 `wp.mat33` 行约定一致），
// 再按上游顺序加 `pos`。
//
// 容量边界（本实现写死；超出时返回 0 → 由 `g5_multicontact` 回退单接触）：
// - 单顶点相邻 polygon 数（= `polymapnum[vertadr+v]`，上游 `nmeshdegmax`）≤
//   `G5_MC_MESH_DEG_MAX`(10)，与 `contact.cu` 的 `n1`/`n2`/`endvert` scratch 一致；
// - 单个 polygon 顶点数（`polyvertnum`，上游 `npolygonmax`）≤ `G5_MC_MESH_FACE_MAX`(20，
//   A1 2026-10-31 由 16 提升)，与 `face1`/`face2` scratch 一致。
// 本片 fixture（cube）的 degree=3、polygon 顶点=4，远低于上界；一般 mesh 超出上界的组合
// 记未覆盖，不改变判据。
#pragma once

// 上游 `_intersect1`（L1591-1602）：求两个整数区间的最多 2 个公共值，返回个数。
// `res` 长度为 2（对应上游 `wp.vec2i`）；按区间顺序扫描，最多到 2 即返回。
__device__ __forceinline__ int g5_mc_intersect1(const int* a1, const int* a2, int start1,
                                                 int start2, int len1, int len2, int* res) {
    int count = 0;
    for (int i = start1; i < start1 + len1; ++i) {
        for (int j = start2; j < start2 + len2; ++j) {
            if (a1[i] == a2[j]) {
                res[count] = a1[i];
                count += 1;
                if (count == 2) {
                    return 2;
                }
            }
        }
    }
    return count;
}

// 上游 `_intersect2`（L1605-1616）：把已收集的最多 2 个值与第三个整数区间求交。
__device__ __forceinline__ int g5_mc_intersect2(const int* a1, const int* a2, int start2, int len1,
                                                 int len2, int* res) {
    int count = 0;
    for (int i = 0; i < len1; ++i) {
        for (int j = start2; j < start2 + len2; ++j) {
            if (a1[i] == a2[j]) {
                res[count] = a1[i];
                count += 1;
                if (count == 2) {
                    return 2;
                }
            }
        }
    }
    return count;
}

// 上游 `_mesh_normals`（L1620-1686）：由最多 3 个 mesh 顶点（局部顶点号）恢复可能的 polygon
// 法线。`vertadr` = 该 geom 的 mesh_vert 起点；`polyadr` = `mesh_polyadr[dataid]`
// （逐 mesh 的首个 polygon 地址）。返回写出的法线数；0 表示无法恢复。
__device__ __forceinline__ int g5_mc_mesh_normals(int feature_dim, const int* feature_index,
                                                   const float* mat, int vertadr, int polyadr,
                                                   const g5_mc_mesh_view& mesh, v3* normal_out,
                                                   int* index_out) {
    int v1 = feature_index[0];
    int v2 = feature_index[1];
    int v3i = feature_index[2];

    if (feature_dim == 3) {
        int v1_adr = mesh.polymapadr[vertadr + v1];
        int v1_num = mesh.polymapnum[vertadr + v1];

        int v2_adr = mesh.polymapadr[vertadr + v2];
        int v2_num = mesh.polymapnum[vertadr + v2];

        int v3_adr = mesh.polymapadr[vertadr + v3i];
        int v3_num = mesh.polymapnum[vertadr + v3i];

        int edgeset[2];
        int n = g5_mc_intersect1(mesh.polymap, mesh.polymap, v1_adr, v2_adr, v1_num, v2_num,
                                 edgeset);
        if (n == 0) {
            return 0;
        }
        int faceset[2];
        n = g5_mc_intersect2(edgeset, mesh.polymap, v3_adr, n, v3_num, faceset);
        if (n == 0) {
            return 0;
        }

        // three vertices on mesh define a unique face
        normal_out[0] = mul_mat3_vec(mat, g5_mc_mesh_poly_normal(mesh, polyadr + faceset[0]));
        index_out[0] = faceset[0];
        return 1;
    }

    if (feature_dim == 2) {
        int v1_adr = mesh.polymapadr[vertadr + v1];
        int v1_num = mesh.polymapnum[vertadr + v1];

        int v2_adr = mesh.polymapadr[vertadr + v2];
        int v2_num = mesh.polymapnum[vertadr + v2];

        // up to two faces as two vertices define an edge
        int edgeset[2];
        int n = g5_mc_intersect1(mesh.polymap, mesh.polymap, v1_adr, v2_adr, v1_num, v2_num,
                                 edgeset);
        if (n == 0) {
            return 0;
        }
        for (int i = 0; i < n; ++i) {
            normal_out[i] = mul_mat3_vec(mat, g5_mc_mesh_poly_normal(mesh, polyadr + edgeset[i]));
            index_out[i] = edgeset[i];
        }
        return n;
    }

    if (feature_dim == 1) {
        int v1_adr = mesh.polymapadr[vertadr + v1];
        int v1_num = mesh.polymapnum[vertadr + v1];
        if (v1_num > G5_MC_MESH_DEG_MAX) {
            return 0;
        }
        for (int i = 0; i < v1_num; ++i) {
            int index = mesh.polymap[v1_adr + i];
            normal_out[i] = mul_mat3_vec(mat, g5_mc_mesh_poly_normal(mesh, polyadr + index));
            index_out[i] = index;
        }
        return v1_num;
    }
    return 0;
}

// 上游 `_mesh_edge_normals`（L1690-1734）：由最多 2 个顶点恢复可能的边方向与端点。
__device__ __forceinline__ int g5_mc_mesh_edge_normals(int dim, const float* mat, v3 pos,
                                                       int vertadr, int polyadr,
                                                       const g5_mc_mesh_view& mesh, v3 v1, v3 v2,
                                                       int v1i, v3* normal_out, v3* endvert_out) {
    // only one edge
    if (dim == 2) {
        endvert_out[0] = v2;
        normal_out[0] = v3_normalize(v3_sub(v2, v1));
        return 1;
    }

    if (dim == 1) {
        int v1_adr = mesh.polymapadr[vertadr + v1i];
        int v1_num = mesh.polymapnum[vertadr + v1i];
        if (v1_num > G5_MC_MESH_DEG_MAX) {
            return 0;
        }

        // loop through all faces with vertex v1
        for (int i = 0; i < v1_num; ++i) {
            int idx = mesh.polymap[v1_adr + i];
            int adr = mesh.polyvertadr[polyadr + idx];
            int nvert = mesh.polyvertnum[polyadr + idx];
            // find previous vertex in polygon to form edge
            for (int j = 0; j < nvert; ++j) {
                if (mesh.polyvert[adr + j] == v1i) {
                    int k = (j == 0) ? nvert - 1 : j - 1;
                    endvert_out[i] = v3_add(
                        mul_mat3_vec(mat,
                                     g5_mesh_vertex(mesh.vert, vertadr + mesh.polyvert[adr + k])),
                        pos);
                    normal_out[i] = v3_normalize(v3_sub(endvert_out[i], v1));
                }
            }
        }
        return v1_num;
    }
    return 0;
}

// 上游 `_mesh_face`（L1926-1948）：由 polygon 索引恢复 mesh 面的世界顶点（逆序）。
__device__ __forceinline__ int g5_mc_mesh_face(const float* mat, v3 pos, int vertadr, int polyadr,
                                                const g5_mc_mesh_view& mesh, int idx,
                                                v3* face_out) {
    int adr = mesh.polyvertadr[polyadr + idx];
    int nvert = mesh.polyvertnum[polyadr + idx];
    if (nvert > G5_MC_MESH_FACE_MAX) {
        return 0;
    }
    int j = 0;
    for (int i = nvert - 1; i >= 0; --i) {
        face_out[j] = v3_add(mul_mat3_vec(mat, g5_mesh_vertex(mesh.vert, vertadr +
                                                                          mesh.polyvert[adr + i])),
                             pos);
        j += 1;
    }
    return nvert;
}
// G10@P5 G5：multicontact 的多边形裁剪设备版（上游 `_polygon_clip` 及其平面/取点助手）。
// Adapted from frozen MJWarp collision_gjk.py L1487-1528（`_area4` / `_polygon_quad`）、
// L2027-2195（`_plane_normal` / `_halfspace` / `_plane_intersect` / `_witness_on_face` /
// `_polygon_clip`）, revision 71da24d. Apache-2.0.
//
// `_set_edge`（L2199）属几何恢复助手，放在 `multicontact_geom.cu`；本文件只做裁剪。
// 数值纪律同 geom 片：全 f32、上游操作数顺序、不做代数化简。
//
// 容量契约（调用方 `multicontact.cu` 的本地 scratch；A1，2026-10-31）：`plane_normal` 20 个 v3、
// `plane_dist` 20 个 float（npolygonmax=20 时的最大 nface1）、`polygon_out`/`clipped_out` 各 40 个 v3
// （上游 npolygonmax=20 的两倍上界，见 L2094 注释）。
#pragma once

// 上游 L50：`INTERSECT_TOL`（平面与线段相交的参数容差，逐字 0.0000003）。
#define G5_MC_INTERSECT_TOL 0.0000003f  // 0x34A10FB0
// 上游 L30：`FLOAT_MAX`（`_plane_intersect` 的“平行/无交点”哨兵）。
#define G5_MC_FLOAT_MAX 1.0e30f  // 0x7149F2CA

// 上游 `_area4`（L1487）：空间中四点组成的四边形面积。
__device__ __forceinline__ float g5_mc_area4(v3 a, v3 b, v3 c, v3 d) {
    return 0.5f * v3_length(v3_cross(v3_sub(a, c), v3_sub(b, d)));
}

// 上游 `_polygon_quad`（L1493）：凸多边形（npolygon > 4）中面积最大的四边形索引。
// 上游用 `res` 记住最近一次改进；未改进时不回写，故四个下标只在 m 提升处更新。
__device__ __forceinline__ void g5_mc_polygon_quad(const v3* polygon, int npolygon, int* out) {
    int b = 1;
    int c = 2;
    int d = 3;
    out[0] = 0;
    out[1] = b;
    out[2] = c;
    out[3] = d;
    float m = g5_mc_area4(polygon[0], polygon[b], polygon[c], polygon[d]);
    for (int a = 0; a < npolygon; ++a) {
        while (true) {
            float m_next =
                g5_mc_area4(polygon[a], polygon[b], polygon[c], polygon[(d + 1) % npolygon]);
            if (m_next <= m) break;
            m = m_next;
            d = (d + 1) % npolygon;
            out[0] = a;
            out[1] = b;
            out[2] = c;
            out[3] = d;
            while (true) {
                float m_next =
                    g5_mc_area4(polygon[a], polygon[b], polygon[(c + 1) % npolygon], polygon[d]);
                if (m_next <= m) break;
                m = m_next;
                c = (c + 1) % npolygon;
                out[0] = a;
                out[1] = b;
                out[2] = c;
                out[3] = d;
                while (true) {
                    float m_next =
                        g5_mc_area4(polygon[a], polygon[(b + 1) % npolygon], polygon[c], polygon[d]);
                    if (m_next <= m) break;
                    m = m_next;
                    b = (b + 1) % npolygon;
                    out[0] = a;
                    out[1] = b;
                    out[2] = c;
                    out[3] = d;
                }
            }
        }
        if (b == a) {
            b = (b + 1) % npolygon;
            if (c == b) {
                c = (c + 1) % npolygon;
                if (d == c) {
                    d = (d + 1) % npolygon;
                }
            }
        }
    }
}

// 上游 `_plane_normal`（L2027）：由边 v1->v2 与方向 n 张成的裁剪平面。
// 返回平面常数 `dot(res, v1)`，平面法线写入 `normal_out`。
__device__ __forceinline__ float g5_mc_plane_normal(v3 v1, v3 v2, v3 n, v3* normal_out) {
    v3 v3v = v3_add(v1, n);
    v3 res = v3_cross(v3_sub(v2, v1), v3_sub(v3v, v1));
    *normal_out = res;
    return v3_dot(res, v1);
}

// 上游 `_halfspace`（L2034）：点 p 是否在过 a、法线 n 的半空间内。
__device__ __forceinline__ bool g5_mc_halfspace(v3 a, v3 n, v3 p) {
    return v3_dot(v3_sub(p, a), n) > -1.0e-10f;
}

// 上游 `_plane_intersect`（L2039）：直线 a + t(b - a) 与平面的交点参数。
__device__ __forceinline__ float g5_mc_plane_intersect(v3 pn, float pd, v3 a, v3 b) {
    float dot = v3_dot(pn, v3_sub(b, a));

    // parallel; no intersection
    if (fabsf(dot) < 1.0e-10f) {
        return G5_MC_FLOAT_MAX;
    }

    return (pd - v3_dot(pn, a)) / dot;
}

// 上游 `_witness_on_face`（L2051）：把裁剪多边形顶点投影成一对 witness 与距离。
__device__ __forceinline__ void g5_mc_witness_on_face(v3 v, v3 p, v3 n, v3 dir, v3* w1, v3* w2,
                                                      float* dist) {
    v3 d = v3_sub(v, p);
    *dist = v3_dot(d, n);
    float abs_dist = fabsf(*dist);
    *w1 = v3_sub(v, v3_scale(dir, abs_dist));
    *w2 = v;
}

// 上游 `_polygon_clip`（L2061）：把 face2 多边形裁剪到 face1 的逐个半空间上，
// 输出 <=4 对 witness 与距离，返回接触数（0 表示退化/无交叠）。
__device__ __forceinline__ int g5_mc_polygon_clip(v3* plane_normal, float* plane_dist,
                                                  const v3* face1, int nface1, const v3* face2,
                                                  int nface2, v3 n, v3 dir, v3* polygon_out,
                                                  v3* clipped_out, v3* witness1, v3* witness2,
                                                  float* dists) {
    for (int i = 0; i < 4; ++i) {
        witness1[i] = v3_make(0.0f, 0.0f, 0.0f);
        witness2[i] = v3_make(0.0f, 0.0f, 0.0f);
        dists[i] = 0.0f;
    }

    // clipping face needs to be at least a triangle
    if (nface1 < 3) {
        return 0;
    }

    // compute plane normal and distance to plane for each vertex
    v3* pn = plane_normal;
    float* pd = plane_dist;
    for (int i = 0; i < nface1 - 1; ++i) {
        pd[i] = g5_mc_plane_normal(face1[i], face1[i + 1], n, &pn[i]);
    }
    pd[nface1 - 1] = g5_mc_plane_normal(face1[nface1 - 1], face1[0], n, &pn[nface1 - 1]);

    // reserve 2 * max_sides as max sides for a clipped polygon
    int npolygon = nface2;
    int nclipped = 0;

    for (int i = 0; i < nface2; ++i) {
        polygon_out[i] = face2[i];
    }

    // clip the polygon by one edge e at a time
    v3* polygon = polygon_out;
    v3* clipped = clipped_out;
    for (int e = 0; e < nface1; ++e) {
        for (int i = 0; i < npolygon; ++i) {
            // get edge PQ of the polygon
            v3 P = polygon[i];
            v3 Q = polygon[(i + 1) % npolygon];

            // determine if P and Q are in the halfspace of the clipping edge
            bool inside1 = g5_mc_halfspace(face1[e], pn[e], P);
            bool inside2 = g5_mc_halfspace(face1[e], pn[e], Q);

            // PQ entirely outside the clipping edge, skip
            if (!inside1 && !inside2) {
                continue;
            }

            // edge PQ is inside the clipping edge, add Q
            if (inside1 && inside2) {
                clipped[nclipped] = Q;
                nclipped += 1;
                continue;
            }

            // add new vertex to clipped polygon where PQ intersects the clipping edge
            float t = g5_mc_plane_intersect(pn[e], pd[e], P, Q);
            if (t > -G5_MC_INTERSECT_TOL && t < 1.0f + G5_MC_INTERSECT_TOL) {
                t = fminf(fmaxf(t, 0.0f), 1.0f);  // 上游 wp.clamp(t, 0.0, 1.0)
                clipped[nclipped] = v3_add(P, v3_scale(v3_sub(Q, P), t));
                nclipped += 1;
            }

            // add Q as PQ is now back inside the clipping edge
            if (inside2) {
                clipped[nclipped] = Q;
                nclipped += 1;
            }
        }

        // swap clipped and polygon
        v3* tmp = polygon;
        polygon = clipped;
        clipped = tmp;
        npolygon = nclipped;
        nclipped = 0;
    }

    // prune out vertices with positive distance from the face
    int m = npolygon;
    npolygon = 0;
    for (int i = 0; i < m; ++i) {
        if (v3_dot(v3_sub(polygon[i], face1[0]), n) <= 0.0f) {
            if (npolygon != i) {
                polygon[npolygon] = polygon[i];
            }
            npolygon += 1;
        }
    }

    if (npolygon < 1) {
        return 0;
    }

    // if the face is an edge, remove potential duplicates
    if (nface2 == 2 && npolygon > 2) {
        int best1 = 0;
        int best2 = 1;
        float max_d = 0.0f;
        for (int i = 0; i < npolygon; ++i) {
            v3 polygon_out_i = polygon[i];
            for (int j = i + 1; j < npolygon; ++j) {
                v3 diff = v3_sub(polygon[j], polygon_out_i);
                float d2 = v3_dot(diff, diff);
                if (d2 > max_d) {
                    max_d = d2;
                    best1 = i;
                    best2 = j;
                }
            }
        }

        g5_mc_witness_on_face(polygon[best1], face1[0], n, dir, &witness1[0], &witness2[0],
                              &dists[0]);
        g5_mc_witness_on_face(polygon[best2], face1[0], n, dir, &witness1[1], &witness2[1],
                              &dists[1]);

        return 2;
    }

    if (npolygon > 4) {
        int quad[4];
        g5_mc_polygon_quad(polygon, npolygon, quad);
        for (int i = 0; i < 4; ++i) {
            g5_mc_witness_on_face(polygon[quad[i]], face1[0], n, dir, &witness1[i], &witness2[i],
                                  &dists[i]);
        }
        return 4;
    }

    // no pruning needed
    for (int i = 0; i < npolygon; ++i) {
        g5_mc_witness_on_face(polygon[i], face1[0], n, dir, &witness1[i], &witness2[i], &dists[i]);
    }
    return npolygon;
}
// G10@P5 G5/M2：multicontact 主函数设备版（上游 `multicontact()`）。
// Adapted from frozen MJWarp collision_gjk.py L2216-2451, revision 71da24d. Apache-2.0.
//
// 与上游逐行对应：`witness1[0]=x1`、`witness2[0]=x2` → `_feature_dim` → 面/边法线 →
// `_aligned_faces` / `_aligned_face_edge` → 面恢复（`_box_face`/`_mesh_face`/`_cylinder_face`/
// `_set_edge`）→ `_polygon_clip`。返回 `n, witness1, witness2, dists`。
//
// M2：mesh 分支按上游 L2245-2262（mesh 数组视图）、L2278-2291 / L2297-2310（`_mesh_normals`）、
// L2326-2345 / L2360-2379（`_mesh_edge_normals`）、L2400-2412 / L2422-2434（`_mesh_face`）
// 接入（mesh 助手在 `multicontact_mesh.cu`）。sidecar 缺失（9 数组全空 / `mesh_polyadr < 0`）
// 时回退单接触，与 M0a 的 null sidecar 语义一致。box/cylinder 路径逐字保持不变。
//
// 调用方（`contact.cu`）保证 scratch 容量（A1，2026-10-31）：`polygon`/`clipped` 各 40 个 v3、
// `plane_normal` 20、`plane_dist` 20、`idx1`/`idx2`/`n1`/`n2`/`endvert` 各 10、`face1`/`face2` 各 20
// （npolygonmax=20；degree 仍 10，U7 未覆盖）。
// 本函数不做动态分配、不用递归。
#pragma once
// EPA face 解包（`g5_get_face_verts`）与 polytope 视图同源，直接复用 polytope 片。

struct g5_mc_result {
    // 契约上 0..4；本实现对“失败/无多接触”统一回退 n=1（witness 为 x1/x2），故实际返回 1..4。
    // 调用方 `contact.cu` 以 `n >= 1` 判定是否采用本结果。
    int n;
    v3 witness1[4];
    v3 witness2[4];
    float dists[4];
};

__device__ __forceinline__ g5_mc_result g5_multicontact(
    v3* polygon, v3* clipped, v3* plane_normal, float* plane_dist, int* idx1, int* idx2, v3* n1,
    v3* n2, v3* endvert, v3* face1, v3* face2, const v3* epa_vert, const int* epa_vert_index,
    unsigned epa_face, v3 x1, v3 x2, const d_geom& geom1, const d_geom& geom2, int geomtype1,
    int geomtype2) {
    g5_mc_result out;
    out.n = 1;
    for (int i = 0; i < 4; ++i) {
        out.witness1[i] = v3_make(0.0f, 0.0f, 0.0f);
        out.witness2[i] = v3_make(0.0f, 0.0f, 0.0f);
        out.dists[i] = 0.0f;
    }
    out.witness1[0] = x1;
    out.witness2[0] = x2;

    // M2：mesh 参与的 pair 先取 mesh polygon 视图（上游 L2245-2262 的共享数组选择；
    // 单 mesh 组合下等价）。sidecar 未提供（9 数组全空 / mesh_polyadr < 0）时回退单接触，
    // 与 M0a 的 null sidecar 语义一致。
    g5_mc_mesh_view mesh;
    mesh.vert = nullptr;
    mesh.polynormal = nullptr;
    mesh.polyvertadr = nullptr;
    mesh.polyvertnum = nullptr;
    mesh.polyvert = nullptr;
    mesh.polymapadr = nullptr;
    mesh.polymapnum = nullptr;
    mesh.polymap = nullptr;
    mesh.has_poly = false;
    if (geomtype1 == G5_MESH || geomtype2 == G5_MESH) {
        const d_geom& mesh_geom = (geomtype1 == G5_MESH) ? geom1 : geom2;
        mesh = g5_mc_mesh_view_of(mesh_geom);
        if (!mesh.has_poly || mesh_geom.mesh_polyadr < 0) {
            return out;
        }
    }

    int face[3];
    g5_get_face_verts(epa_face, face);

    // get dimensions of features of geoms 1 and 2
    int feature_index1[3];
    int feature_index2[3];
    v3 feature_vertex1[3];
    v3 feature_vertex2[3];
    int nface1 =
        g5_mc_feature_dim(face, epa_vert_index, epa_vert, 0, feature_index1, feature_vertex1);
    int nface2 =
        g5_mc_feature_dim(face, epa_vert_index, epa_vert, 1, feature_index2, feature_vertex2);

    v3 dir = v3_sub(x2, x1);
    v3 dir_neg = v3_neg(dir);

    // get all possible face normals for each geom
    int nnorms1 = 0;
    int nnorms2 = 0;
    if (geomtype1 == G5_BOX) {
        nnorms1 = g5_mc_box_normals(nface1, feature_index1, geom1.rot, dir_neg, n1, idx1);
    } else if (geomtype1 == G5_MESH) {
        nnorms1 = g5_mc_mesh_normals(nface1, feature_index1, geom1.rot, geom1.vertadr,
                                     geom1.mesh_polyadr, mesh, n1, idx1);
    } else if (geomtype1 == G5_CYLINDER) {
        nnorms1 = g5_mc_cylinder_normals(nface1, feature_index1, geom1.rot, n1, idx1);
    }

    if (geomtype2 == G5_BOX) {
        nnorms2 = g5_mc_box_normals(nface2, feature_index2, geom2.rot, dir, n2, idx2);
    } else if (geomtype2 == G5_MESH) {
        nnorms2 = g5_mc_mesh_normals(nface2, feature_index2, geom2.rot, geom2.vertadr,
                                     geom2.mesh_polyadr, mesh, n2, idx2);
    } else if (geomtype2 == G5_CYLINDER) {
        nnorms2 = g5_mc_cylinder_normals(nface2, feature_index2, geom2.rot, n2, idx2);
    }

    // determine if any two face normals match
    int is_edge_contact_geom1 = 0;
    int is_edge_contact_geom2 = 0;
    int res[2];
    int nres = g5_mc_aligned_faces(n1, nnorms1, n2, nnorms2, res);
    if (!nres) {
        // check if edge-face collision
        if (nface1 < 3 && nface1 <= nface2) {
            nnorms1 = 0;
            if (geomtype1 == G5_BOX) {
                nnorms1 = g5_mc_box_edge_normals(nface1, geom1.rot, geom1.pos, geom1.size,
                                                feature_vertex1[0], feature_vertex1[1],
                                                feature_index1[0], n1, endvert);
            } else if (geomtype1 == G5_MESH) {
                nnorms1 = g5_mc_mesh_edge_normals(nface1, geom1.rot, geom1.pos, geom1.vertadr,
                                                  geom1.mesh_polyadr, mesh, feature_vertex1[0],
                                                  feature_vertex1[1], feature_index1[0], n1,
                                                  endvert);
            } else if (geomtype1 == G5_CYLINDER) {
                nnorms1 = g5_mc_cylinder_edge_normals(nface1, geom1.rot, geom1.size,
                                                      feature_vertex1[0], feature_index1[0], n1,
                                                      endvert);
            }
            nres = g5_mc_aligned_face_edge(n1, nnorms1, n2, nnorms2, dir, res);
            if (!nres) {
                return out;
            }
            is_edge_contact_geom1 = 1;
        }
        // check if face-edge collision
        else if (nface2 < 3) {
            nnorms2 = 0;
            if (geomtype2 == G5_BOX) {
                nnorms2 = g5_mc_box_edge_normals(nface2, geom2.rot, geom2.pos, geom2.size,
                                                feature_vertex2[0], feature_vertex2[1],
                                                feature_index2[0], n2, endvert);
            } else if (geomtype2 == G5_MESH) {
                nnorms2 = g5_mc_mesh_edge_normals(nface2, geom2.rot, geom2.pos, geom2.vertadr,
                                                  geom2.mesh_polyadr, mesh, feature_vertex2[0],
                                                  feature_vertex2[1], feature_index2[0], n2,
                                                  endvert);
            } else if (geomtype2 == G5_CYLINDER) {
                nnorms2 = g5_mc_cylinder_edge_normals(nface2, geom2.rot, geom2.size,
                                                      feature_vertex2[0], feature_index2[0], n2,
                                                      endvert);
            }
            nres = g5_mc_aligned_face_edge(n2, nnorms2, n1, nnorms1, dir_neg, res);
            if (!nres) {
                return out;
            }
            is_edge_contact_geom2 = 1;
        } else {
            // no multi-contact
            return out;
        }
    }

    int i = res[0];
    int j = res[1];

    // recover geom1 matching edge or face
    if (is_edge_contact_geom1) {
        nface1 = g5_mc_set_edge(epa_vert, endvert, face[0], i, 0, face1);
    } else {
        int ind = is_edge_contact_geom2 ? idx1[j] : idx1[i];
        if (geomtype1 == G5_BOX) {
            nface1 = g5_mc_box_face(geom1.rot, geom1.pos, geom1.size, ind, face1);
        } else if (geomtype1 == G5_MESH) {
            nface1 = g5_mc_mesh_face(geom1.rot, geom1.pos, geom1.vertadr, geom1.mesh_polyadr, mesh,
                                     ind, face1);
        } else if (geomtype1 == G5_CYLINDER) {
            nface1 = g5_mc_cylinder_face(geom1.rot, geom1.pos, geom1.size, ind, face1);
        }
    }

    // recover geom2 matching edge or face
    if (is_edge_contact_geom2) {
        nface2 = g5_mc_set_edge(epa_vert, endvert, face[0], i, 1, face2);
    } else {
        if (geomtype2 == G5_BOX) {
            nface2 = g5_mc_box_face(geom2.rot, geom2.pos, geom2.size, idx2[j], face2);
        } else if (geomtype2 == G5_MESH) {
            nface2 = g5_mc_mesh_face(geom2.rot, geom2.pos, geom2.vertadr, geom2.mesh_polyadr, mesh,
                                     idx2[j], face2);
        } else if (geomtype2 == G5_CYLINDER) {
            nface2 = g5_mc_cylinder_face(geom2.rot, geom2.pos, geom2.size, idx2[j], face2);
        }
    }

    // M2：mesh 面/边超出 scratch 容量时助手返回 0 → 无法恢复 feature，回退单接触
    // （box/cylinder 助手恒返回 >=1，故对既有路径无行为变化）。
    if (nface1 == 0 || nface2 == 0) {
        return out;
    }

    // 裁剪结果的 witness 先落到本地，便于 `_polygon_clip` 失败时保留 x1/x2 单接触回退。
    v3 clip_w1[4];
    v3 clip_w2[4];
    float clip_d[4];
    int nclip = 0;
    int swapped = 0;

    // face1 is an edge; clip face1 against face2
    if (is_edge_contact_geom1) {
        nclip = g5_mc_polygon_clip(plane_normal, plane_dist, face2, nface2, face1, nface1, n2[j],
                                   v3_neg(n2[j]), polygon, clipped, clip_w1, clip_w2, clip_d);
        // the faces were flipped in calling _polygon_clip so we need to flip them back
        swapped = 1;
    } else if (is_edge_contact_geom2) {
        // face2 is an edge; clip face2 against face1
        nclip = g5_mc_polygon_clip(plane_normal, plane_dist, face1, nface1, face2, nface2, n1[j],
                                   v3_neg(n1[j]), polygon, clipped, clip_w1, clip_w2, clip_d);
    } else {
        // face-face collision
        nclip = g5_mc_polygon_clip(plane_normal, plane_dist, face1, nface1, face2, nface2, n1[i],
                                   n2[j], polygon, clipped, clip_w1, clip_w2, clip_d);
    }

    // 上游调用方在 ncollision < 1 时回退 EPA 单接触（witness 用 x1/x2）。
    if (nclip < 1) {
        return out;
    }

    out.n = nclip;
    for (int k = 0; k < nclip; ++k) {
        out.witness1[k] = swapped ? clip_w2[k] : clip_w1[k];
        out.witness2[k] = swapped ? clip_w1[k] : clip_w2[k];
        out.dists[k] = clip_d[k];
    }
    return out;
}
// G10@P5 G5：接触写入、nacon/naccd 容量截断与 OverflowType 汇总设备版。
// Adapted from frozen MJWarp `collision_core.py::write_contact` 与
// `collision_convex.py::eval_ccd_write_contact` / `ccd_hfield_kernel`, revision 71da24d.
// Apache-2.0.
//
// 上游实测口径（`io.py`）：`nacon` 是**全局**单计数器（`nacon: array(1)`），接触数组行全局编号
// （长度 `naconmax`），world 归属靠 `contact_worldid` 字段，**没有逐 world 行分段**
// （`naconmax` 总行 = `nconmax * nworld` 只是默认换算）。`naccdmax` 同样是全局总量，
// `nccd` 计数按 `geomgeomid` 分组、容量判定 `ccdid >= naccdmax`。
#pragma once

// 上游 `ContactType.CONSTRAINT` / `ContactType.SENSOR`。
#define G5_CONTACT_TYPE_CONSTRAINT (1u << 0)
#define G5_CONTACT_TYPE_SENSOR (1u << 1)
// 上游 `vec5` 宽度（friction/solimp）。
#define G5_CONTACT_VEC5 5
// 上游 `wp.vec2` 宽度（solref/solreffriction）。
#define G5_CONTACT_VEC2 2

// 上游 `contact_*` 输出数组视图（全局行编号，容量由调用方给出）。
struct d_contact_out {
  float* dist;
  v3* pos;
  // [naconmax][9]，行优先 3×3 frame。
  float* frame;
  float* includemargin;
  float* friction;        // [naconmax][5]
  float* solref;          // [naconmax][2]
  float* solreffriction;  // [naconmax][2]
  float* solimp;          // [naconmax][5]
  int* dim;
  int* geom;              // [naconmax][2]
  // [naconmax][efc_address_width]；初始写 -1，真实 efc 编号由约束管线填。
  int* efc_address;
  int* worldid;
  int* type;
  int* geomcollisionid;
  float* adhesion;
  // 上游 `nacon`：全局检测到的接触数（可超过 naconmax，用于诊断/传感器）。
  int* nacon;
  int efc_address_width;
};

// 上游 `contact_params` / `contact_material_params` 的单对结果。
struct d_contact_params {
  // 上游 `contact_margin_gap`：pair 覆盖或两侧 geom 之和。
  float margin;
  float gap;
  int condim;
  float friction[G5_CONTACT_VEC5];
  float solref[G5_CONTACT_VEC2];
  float solreffriction[G5_CONTACT_VEC2];
  float solimp[G5_CONTACT_VEC5];
  float adhesion;
  int geoms[2];
  // 上游 `pairid`：`pairid[0]` 为 pair 匹配号，`pairid[1]` 为 collision sensor 号。
  int pairid[2];
  int worldid;
};

// 上游 `write_contact`：原子分配行号；只有 `cid < naconmax` 才写数组（不越界写）。
// 返回 1 表示 active 接触（上游语义），0 表示不活跃/被截断。
__device__ __forceinline__ int g5_write_contact(const d_contact_params& params, int naconmax,
                                                int id_, float dist_in, v3 pos_in,
                                                const float* frame_in, d_contact_out* out) {
  bool active = (dist_in < params.margin) ||
                ((params.adhesion != 0.0f) && (dist_in < params.margin + params.gap));
  bool detected = dist_in < params.margin + params.gap;
  int condim = ((params.adhesion != 0.0f) && (dist_in >= params.margin)) ? 1 : params.condim;

  // 上游：跳过不检测的 pair（非 collision sensor 且 pairid[0] == -2）。
  if (((params.pairid[0] == -2) || !detected) && (params.pairid[1] == -1)) {
    return 0;
  }

  unsigned contact_type = 0u;
  if (params.pairid[0] >= -1 && detected) contact_type |= G5_CONTACT_TYPE_CONSTRAINT;
  if (params.pairid[1] >= 0) contact_type |= G5_CONTACT_TYPE_SENSOR;

  // 上游先 atomic_add 再判容量：超容量时计数仍递增，行不写。
  int cid = atomicAdd(out->nacon, 1);
  if (cid < naconmax) {
    out->dist[cid] = dist_in;
    out->pos[cid] = pos_in;
    for (int k = 0; k < 9; ++k) out->frame[cid * 9 + k] = frame_in[k];
    out->geom[cid * 2] = params.geoms[0];
    out->geom[cid * 2 + 1] = params.geoms[1];
    out->worldid[cid] = params.worldid;
    out->includemargin[cid] = params.margin;
    out->dim[cid] = condim;
    for (int k = 0; k < G5_CONTACT_VEC5; ++k) {
      out->friction[cid * G5_CONTACT_VEC5 + k] = params.friction[k];
      out->solimp[cid * G5_CONTACT_VEC5 + k] = params.solimp[k];
    }
    for (int k = 0; k < G5_CONTACT_VEC2; ++k) {
      out->solref[cid * G5_CONTACT_VEC2 + k] = params.solref[k];
      out->solreffriction[cid * G5_CONTACT_VEC2 + k] = params.solreffriction[k];
    }
    out->adhesion[cid] = params.adhesion;
    out->type[cid] = (int)contact_type;
    // 上游 `contact_geomcollisionid_out[cid] = id_`（对内的接触序号）。
    out->geomcollisionid[cid] = id_;
    for (int k = 0; k < out->efc_address_width; ++k) {
      out->efc_address[cid * out->efc_address_width + k] = -1;
    }
    return active ? 1 : 0;
  }
  return 0;
}

// 上游 `nccd`/`naccdmax` 容量门：按 `geomgeomid` 分组原子申请 EPA 槽；
// 超容量只置 `OverflowType.CCD`，不写接触（不越界使用 EPA 缓冲）。
__device__ __forceinline__ bool g5_claim_ccd_slot(int geomgeomid, int* nccd, int naccdmax,
                                                  int worldid, unsigned* overflow_out) {
  int ccdid = atomicAdd(nccd + geomgeomid, 1);
  if (ccdid >= naccdmax) {
    atomicOr(overflow_out + worldid, G5_OVERFLOW_CCD);
    return false;
  }
  return true;
}

// 上游 `eval_ccd_write_contact` 的单对输入。
struct d_contact_pair_input {
  // 世界系几何视图（内部做原点平移）。
  d_geom geom1;
  d_geom geom2;
  int geomtype1;
  int geomtype2;
  // `opt.ccd_tolerance`、`opt.ccd_iterations`、`epa_iterations`。
  float tolerance;
  int gjk_iterations;
  int epa_iterations;
  // 上游 `pairid[1] >= 0`：collision sensor 对（cutoff=1e32、跳过 gap 早退）。
  bool collision_sensor;
  // 上游 `naccdmax` 与 `nccd[geomgeomid]` 计数器（全局容量）。
  int naccdmax;
  int geomgeomid;
  // `m.opt.warn_overflow & OverflowType.*`；设备侧不打印，只汇总位。
  unsigned* overflow_out;
};

// 上游 `eval_ccd_write_contact`：GJK → （需 EPA 时申请 naccd 槽）EPA
// → gap 早退 → margin 修正 → （可选的 multicontact）→ frame/witness → 逐接触写入。
// `use_multiccd` 对应上游 kernel 构建期的静态特化（不同入口传不同值）。
__device__ __forceinline__ bool g5_mc_supported_type(int geom_type) {
  return geom_type == G5_BOX || geom_type == G5_MESH || geom_type == G5_CYLINDER;
}

__device__ __forceinline__ void g5_ccd_pair_write(const d_contact_pair_input& in,
                                                  const d_contact_params& params, int naconmax,
                                                  bool use_multiccd, int* nccd, d_polytope* pt,
                                                  d_contact_out* out) {
  d_geom geom1 = in.geom1;
  d_geom geom2 = in.geom2;
  geom1.margin = params.margin;
  geom2.margin = params.margin;

  float cutoff = in.collision_sensor ? 1.0e32f : params.gap;

  // 上游：原点平移到 geom1 位置（平移不变性下的数值精度）。
  v3 origin = geom1.pos;
  geom1.pos = v3_make(0.0f, 0.0f, 0.0f);
  geom2.pos = v3_sub(geom2.pos, origin);

  d_gjk_phase phase;
  g5_gjk_phase(in.tolerance, cutoff, in.gjk_iterations, geom1, geom2, in.geomtype1,
               in.geomtype2, geom1.pos, geom2.pos, &phase);

  float dist = phase.dist;
  int ncontact = phase.ncontact;
  v3 w1 = phase.x1;
  v3 w2 = phase.x2;
  int multiccd_idx = -1;
  bool has_polytope = false;

  if (phase.needs_epa) {
    if (!g5_claim_ccd_slot(in.geomgeomid, nccd, in.naccdmax, params.worldid,
                           in.overflow_out)) {
      return;
    }
    d_epa_phase epa;
    g5_epa_phase(in.tolerance, in.epa_iterations, phase.result, phase.geom1, phase.geom2,
                 in.geomtype1, in.geomtype2, pt, &epa);
    dist = epa.dist;
    ncontact = epa.ncontact;
    w1 = epa.x1;
    w2 = epa.x2;
    multiccd_idx = epa.multiccd_idx;
    has_polytope = epa.has_polytope;
    if (epa.overflow != 0u) atomicOr(in.overflow_out + params.worldid, epa.overflow);
    // 上游 `eval_ccd_write_contact` 只汇总 EPA 阶段的溢出位（`EPA_HORIZON`）；
    // `CCD` 位只在上面 `nccd >= naccdmax` 的真实容量截断时置，不由 `FLOAT_MAX` 推断。
  }

  if (dist >= params.gap && !in.collision_sensor) return;

  // 上游：CCD 在 margin 膨胀形状上工作，dist 修正回真实表面距离。
  dist += params.margin;

  // 上游 multicontact 分支：`(use_multiccd or (BOX,BOX))` 且两侧类型都在
  // {BOX, MESH, CYLINDER}；`multiccd_idx > -1` 时用 EPA 最终穿透面生成 <=4 个接触。
  int ncontacts = ncontact;
  v3 witness1[4];
  v3 witness2[4];
  float dists[4];
  witness1[0] = w1;
  witness2[0] = w2;
  dists[0] = dist;
  bool want_multi = use_multiccd ||
                    (in.geomtype1 == G5_BOX && in.geomtype2 == G5_BOX);
  if (want_multi && has_polytope && multiccd_idx > -1 &&
      g5_mc_supported_type(in.geomtype1) && g5_mc_supported_type(in.geomtype2)) {
    // 上游 convex_narrowphase 的 scratch 容量上界：npolygonmax=20（A1，2026-10-31；含 cylinder）、
    // nmeshdegmax 取 10 的安全上界（本片只实现 box/cylinder/mesh 分支）。
    v3 polygon[40];
    v3 clipped[40];
    v3 pnormal[20];
    float pdist[20];
    int idx1[10];
    int idx2[10];
    v3 n1[10];
    v3 n2[10];
    v3 endvert[10];
    v3 face1[20];
    v3 face2[20];
    g5_mc_result mc = g5_multicontact(polygon, clipped, pnormal, pdist, idx1, idx2, n1, n2,
                                      endvert, face1, face2, pt->vert, pt->vert_index,
                                      pt->face[multiccd_idx], w1, w2, phase.geom1, phase.geom2,
                                      in.geomtype1, in.geomtype2);
    if (mc.n >= 1) {
      ncontacts = mc.n;
      for (int i = 0; i < 4; ++i) {
        witness1[i] = mc.witness1[i];
        witness2[i] = mc.witness2[i];
        dists[i] = mc.dists[i];
      }
    }
  }

  // 上游：重叠时 witness1 越过 witness2，方向按符号取反（frame 只算一次，全部接触共用）。
  float frame[9];
  if (dist <= params.margin) {
    make_frame(v3_sub(witness1[0], witness2[0]), frame);
  } else {
    make_frame(v3_sub(witness2[0], witness1[0]), frame);
  }

  // 上游 `for i in range(ncollision)`：n>1 时用 multicontact 的 dists[i]，否则用 EPA 的 dist；
  // 位置统一为 `0.5*(witness1[i]+witness2[i])+origin`，`id_` = 接触序号。
  for (int i = 0; i < ncontacts; ++i) {
    float contact_dist = (ncontacts > 1) ? dists[i] : dist;
    v3 pos = v3_add(v3_scale(v3_add(witness1[i], witness2[i]), 0.5f), origin);
    g5_write_contact(params, naconmax, i, contact_dist, pos, frame, out);
  }
}

// hfield 路径的接触写入：上游 `ccd_hfield_kernel` 在选好 0..3 个接触后逐个
// `write_contact`（`id_` = 0..3）；溢出位（HFIELD/CCD/EPA_HORIZON）在写接触前汇总。
__device__ __forceinline__ void g5_hfield_write_contacts(const d_hfield_result& result,
                                                         const d_contact_params& params,
                                                         int naconmax, unsigned* overflow_out,
                                                         d_contact_out* out) {
  if (result.overflow != 0u) atomicOr(overflow_out + params.worldid, result.overflow);
  for (int i = 0; i < result.ncontact; ++i) {
    g5_write_contact(params, naconmax, i, result.contact[i].dist, result.contact[i].pos,
                     result.contact[i].frame, out);
  }
}
// G10@P5 G5：上游 primitive narrowphase 数值内核（sphere-box / capsule-box）设备版。
// Adapted from frozen MJWarp `collision_primitive_core.py::sphere_box` / `capsule_box`,
// revision 71da24d. Apache-2.0.
//
// 逐表达式按 f32 转写（含循环展开顺序、整数编码与 `safe_div`/`normalize_with_norm`
// 语义）；上层 wrapper 与接触写入见 `primitive_wrapper.cu`。

// 上游 `types.MJ_MAXVAL`（`mujoco.mjMAXVAL = 1e10`）：capsule-box 未生成第二接触时使用。
#define G5_PRIMITIVE_MAXVAL 1.0e10f
// 上游 `types.MJ_MINVAL`（`mujoco.mjMINVAL = 1e-15`）。
#define G5_PRIMITIVE_MINVAL 1.0e-15f

// 单条 primitive 接触（对应上游 `sphere_box` 返回的 `(dist, pos, normal)`）。
struct g5_primitive_contact {
  float dist;
  v3 pos;
  v3 normal;
};

// 两条 primitive 接触（对应上游 `capsule_box` 返回的 `vec2` + 两个 `mat23f` 行）。
struct g5_primitive_pair {
  g5_primitive_contact contact[2];
};

// 上游 `math.normalize_with_norm`：范数为零时返回原向量与 0.0（不除零）。
__device__ __forceinline__ void g5_primitive_normalize_with_norm(v3 x, v3* direction,
                                                                 float* norm) {
  const float length = sqrtf(v3_dot(x, x));
  *norm = length;
  if (length == 0.0f) {
    *direction = x;
    return;
  }
  *direction = v3_make(x.x / length, x.y / length, x.z / length);
}

// 上游 `math.safe_div`：`x / where(y != 0.0, y, MJ_MINVAL)`。
__device__ __forceinline__ float g5_primitive_safe_div(float numerator, float denominator) {
  return numerator / (denominator != 0.0f ? denominator : G5_PRIMITIVE_MINVAL);
}

// 上游 `collision_primitive_core.sphere_box`：
//   center = transpose(box_rot) @ (sphere_pos - box_pos)
//   盒内分支按 6 个面距离取最近面；盒外按最近点与最深点中点。
__device__ __forceinline__ g5_primitive_contact g5_sphere_box(
    v3 sphere_pos, float sphere_radius, v3 box_pos, const float* box_rot, v3 box_size) {
  const v3 center = transpose_mul_mat3_vec(box_rot, v3_sub(sphere_pos, box_pos));

  // 上游 `wp.max(-box_size, wp.min(box_size, center))`（分量式）。
  const v3 clamped = v3_make(fmaxf(-box_size.x, fminf(box_size.x, center.x)),
                             fmaxf(-box_size.y, fminf(box_size.y, center.y)),
                             fmaxf(-box_size.z, fminf(box_size.z, center.z)));
  v3 clamped_dir;
  float dist;
  g5_primitive_normalize_with_norm(v3_sub(clamped, center), &clamped_dir, &dist);

  g5_primitive_contact contact;
  if (dist <= G5_PRIMITIVE_MINVAL) {
    // 球心在盒内：枚举 6 个面，取 `closest > face_dist` 的严格更小者（同距取首个）。
    float closest = 2.0f * (box_size.x + box_size.y + box_size.z);
    const float half[3] = {box_size.x, box_size.y, box_size.z};
    const float center_axis[3] = {center.x, center.y, center.z};
    int k = 0;
    for (int i = 0; i < 6; ++i) {
      const float sign = (i % 2) ? 1.0f : -1.0f;
      const float face_dist = fabsf(sign * half[i / 2] - center_axis[i / 2]);
      if (closest > face_dist) {
        closest = face_dist;
        k = i;
      }
    }
    float nearest[3] = {0.0f, 0.0f, 0.0f};
    nearest[k / 2] = (k % 2) ? -1.0f : 1.0f;
    const v3 nearest_v = v3_make(nearest[0], nearest[1], nearest[2]);
    const v3 scaled = v3_scale(nearest_v, sphere_radius - closest);
    const v3 offset = v3_make(scaled.x / 2.0f, scaled.y / 2.0f, scaled.z / 2.0f);
    const v3 pos = v3_add(center, offset);
    contact.dist = -closest - sphere_radius;
    contact.normal = mul_mat3_vec(box_rot, nearest_v);
    contact.pos = v3_add(box_pos, mul_mat3_vec(box_rot, pos));
  } else {
    // 球心在盒外：沿最近面方向把球心推进球半径，取最近点与最深点中点。
    const v3 deepest = v3_add(center, v3_scale(clamped_dir, sphere_radius));
    const v3 pos = v3_scale(v3_add(clamped, deepest), 0.5f);
    contact.dist = dist - sphere_radius;
    contact.normal = mul_mat3_vec(box_rot, clamped_dir);
    contact.pos = v3_add(box_pos, mul_mat3_vec(box_rot, pos));
  }
  return contact;
}

// 上游 `collision_primitive_core.capsule_box`（逐行对照 mjc 实现）：
// 先找与盒面/盒边最近的胶囊端点或边中点，再对 1~2 个球端点调用 `sphere_box`。
__device__ __forceinline__ g5_primitive_pair g5_capsule_box(
    v3 capsule_pos, v3 capsule_axis, float capsule_radius, float capsule_half_length,
    v3 box_pos, const float* box_rot, v3 box_size) {
  const v3 pos = transpose_mul_mat3_vec(box_rot, v3_sub(capsule_pos, box_pos));
  const v3 axis = transpose_mul_mat3_vec(box_rot, capsule_axis);
  const v3 halfaxis = v3_scale(axis, capsule_half_length);
  const int axisdir = (halfaxis.x > 0.0f ? 1 : 0) + 2 * (halfaxis.y > 0.0f ? 1 : 0) +
                      4 * (halfaxis.z > 0.0f ? 1 : 0);

  const float size[3] = {box_size.x, box_size.y, box_size.z};
  const float halfaxis_axis[3] = {halfaxis.x, halfaxis.y, halfaxis.z};
  const float axis_axis[3] = {axis.x, axis.y, axis.z};

  // 最近点跟踪（初值与上游一致）。
  float bestdist = 1.0e32f;
  float bestsegmentpos = -12.0f;
  int cltype = -4;
  int clface = -12;

  // 第一段：考虑盒面最近的配置（胶囊两端点向盒面投影）。
  for (int i = -1; i <= 1; i += 2) {
    const v3 axis_tip = v3_add(pos, v3_scale(halfaxis, (float)i));
    float box_point[3] = {axis_tip.x, axis_tip.y, axis_tip.z};
    int n_out = 0;
    int ax_out = -1;
    for (int j = 0; j < 3; ++j) {
      if (box_point[j] < -size[j]) {
        n_out += 1;
        ax_out = j;
        box_point[j] = -size[j];
      } else if (box_point[j] > size[j]) {
        n_out += 1;
        ax_out = j;
        box_point[j] = size[j];
      }
    }
    if (n_out > 1) continue;
    const v3 box_point_v = v3_make(box_point[0], box_point[1], box_point[2]);
    const v3 delta = v3_sub(box_point_v, axis_tip);
    const float dist = v3_dot(delta, delta);
    if (dist < bestdist) {
      bestdist = dist;
      bestsegmentpos = (float)i;
      cltype = -2 + i;
      clface = ax_out;
    }
  }

  // 第二段：考虑盒边最近的配置（i 编码盒角点，j 为被置零的边轴）。
  int clcorner = -123;
  int cledge = -123;
  float bestboxpos = 0.0f;
  for (int i = 0; i < 8; ++i) {
    for (int j = 0; j < 3; ++j) {
      if ((i & (1 << j)) != 0) continue;
      int c2 = -123;
      float box_pt[3] = {(i & 1) ? size[0] : -size[0], (i & 2) ? size[1] : -size[1],
                         (i & 4) ? size[2] : -size[2]};
      box_pt[j] = 0.0f;
      const v3 box_pt_v = v3_make(box_pt[0], box_pt[1], box_pt[2]);
      v3 dif = v3_sub(box_pt_v, pos);
      float dif_axis[3] = {dif.x, dif.y, dif.z};

      // 胶囊轴与盒边的最近点（两线段参数 x1/x2，clamp 到 [-1, 1]）。
      const float u = -size[j] * dif_axis[j];
      const float v = v3_dot(halfaxis, dif);
      const float ma = size[j] * size[j];
      const float mb = -size[j] * halfaxis_axis[j];
      const float mc = capsule_half_length * capsule_half_length;
      const float det = ma * mc - mb * mb;
      if (fabsf(det) < G5_PRIMITIVE_MINVAL) continue;
      const float idet = 1.0f / det;
      // sX：X=1 表示线段中部，0/2 表示某一端。
      float x1 = (mc * u - mb * v) * idet;
      float x2 = (ma * v - mb * u) * idet;
      int s1 = 1;
      int s2 = 1;
      if (x1 > 1.0f) {
        x1 = 1.0f;
        s1 = 2;
        x2 = g5_primitive_safe_div(v - mb, mc);
      } else if (x1 < -1.0f) {
        x1 = -1.0f;
        s1 = 0;
        x2 = g5_primitive_safe_div(v + mb, mc);
      }
      const bool x2_over = x2 > 1.0f;
      if (x2_over || x2 < -1.0f) {
        if (x2_over) {
          x2 = 1.0f;
          s2 = 2;
          x1 = g5_primitive_safe_div(u - mb, ma);
        } else {
          x2 = -1.0f;
          s2 = 0;
          x1 = g5_primitive_safe_div(u + mb, ma);
        }
        if (x1 > 1.0f) {
          x1 = 1.0f;
          s1 = 2;
        } else if (x1 < -1.0f) {
          x1 = -1.0f;
          s1 = 0;
        }
      }

      dif = v3_sub(dif, v3_scale(halfaxis, x2));
      dif_axis[0] = dif.x;
      dif_axis[1] = dif.y;
      dif_axis[2] = dif.z;
      dif_axis[j] += size[j] * x1;
      const int ct = s1 * 3 + s2;
      const v3 dif_v = v3_make(dif_axis[0], dif_axis[1], dif_axis[2]);
      const float dif_sq = v3_dot(dif_v, dif_v);
      if (dif_sq < bestdist - G5_PRIMITIVE_MINVAL) {
        bestdist = dif_sq;
        bestsegmentpos = x2;
        bestboxpos = x1;
        // ct<6：盒上最近点在边的下端或中部。
        c2 = ct / 6;
        clcorner = i + (1 << j) * c2;
        cledge = j;
        cltype = ct;
      }
    }
  }

  // 平行盒面的方向选择（对应上游 planar 输出帧的 c1 编码）。
  const float p[2] = {pos.x, pos.y};
  const float dd[2] = {halfaxis.x, halfaxis.y};
  const float s[2] = {box_size.x, box_size.y};
  float secondpos = -4.0f;
  const float uu = dd[0] * s[1];
  const float vv = dd[1] * s[0];
  const bool w_neg = dd[0] * p[1] - dd[1] * p[0] < 0.0f;
  float best = -1.0f;
  const float ee1 = uu - vv;
  const float ee2 = uu + vv;
  int c1 = 0;
  if (fabsf(ee1) > best) {
    best = fabsf(ee1);
    c1 = ((ee1 < 0.0f) == w_neg) ? 0 : 3;
  }
  if (fabsf(ee2) > best) {
    best = fabsf(ee2);
    c1 = ((ee2 > 0.0f) == w_neg) ? 1 : 2;
  }

  if (cltype == -4) {
    // 上游无效配置：两个接触距离都是 MJ_MAXVAL，位置/法线为空（wrapper 仍会写两行，
    // `write_contact` 因 `detected == false` 全部跳过）。
    g5_primitive_pair empty;
    empty.contact[0].dist = G5_PRIMITIVE_MAXVAL;
    empty.contact[0].pos = v3_make(0.0f, 0.0f, 0.0f);
    empty.contact[0].normal = v3_make(0.0f, 0.0f, 0.0f);
    empty.contact[1] = empty.contact[0];
    return empty;
  }

  if (cltype >= 0 && cltype / 3 != 1) {
    // 盒角点最近：可能沿胶囊轴再生成第二接触。
    c1 = axisdir ^ clcorner;
    if (c1 != 0 && c1 != 7) {
      int mul;
      if (c1 == 1 || c1 == 2 || c1 == 4) {
        mul = 1;
      } else {
        mul = -1;
        c1 = 7 - c1;
      }
      int ax = 0;
      int ax1 = 0;
      int ax2 = 0;
      if (c1 == 1) {
        ax = 0;
        ax1 = 1;
        ax2 = 2;
      } else if (c1 == 2) {
        ax = 1;
        ax1 = 2;
        ax2 = 0;
      } else if (c1 == 4) {
        ax = 2;
        ax1 = 0;
        ax2 = 1;
      }
      if (axis_axis[ax] * axis_axis[ax] > 0.5f) {
        // 第二点沿盒边。
        const float m = 2.0f * g5_primitive_safe_div(size[ax], fabsf(halfaxis_axis[ax]));
        secondpos = fminf(1.0f - (float)mul * bestsegmentpos, m);
      } else {
        // 第二点沿盒面。
        const float m = 2.0f * fminf(g5_primitive_safe_div(size[ax1], fabsf(halfaxis_axis[ax1])),
                                     g5_primitive_safe_div(size[ax2], fabsf(halfaxis_axis[ax2])));
        secondpos = -fminf(1.0f + (float)mul * bestsegmentpos, m);
      }
      secondpos *= (float)mul;
    }
  } else if (cltype >= 0 && cltype / 3 == 1) {
    // 盒边最近：T 配置（c1 = 2^n）无第二接触，X 配置可能生成第二接触。
    c1 = axisdir ^ clcorner;
    c1 &= 7 - (1 << cledge);
    if (c1 == 1 || c1 == 2 || c1 == 4) {
      int ax1 = 0;
      int ax2 = 0;
      if (cledge == 0) {
        ax1 = 1;
        ax2 = 2;
      }
      if (cledge == 1) {
        ax1 = 2;
        ax2 = 0;
      }
      if (cledge == 2) {
        ax1 = 0;
        ax2 = 1;
      }
      const int ax = cledge;
      if (fabsf(axis_axis[ax1]) > fabsf(axis_axis[ax2])) ax1 = ax2;
      ax2 = 3 - ax - ax1;
      int mul;
      if (c1 & (1 << ax2)) {
        mul = 1;
        secondpos = 1.0f - bestsegmentpos;
      } else {
        mul = -1;
        secondpos = 1.0f + bestsegmentpos;
      }
      float e1 = 2.0f * g5_primitive_safe_div(size[ax2], fabsf(halfaxis_axis[ax2]));
      secondpos = fminf(e1, secondpos);
      float e2;
      if (((axisdir & (1 << ax)) != 0) == ((c1 & (1 << ax2)) != 0)) {
        e2 = 1.0f - bestboxpos;
      } else {
        e2 = 1.0f + bestboxpos;
      }
      e1 = size[ax] * g5_primitive_safe_div(e2, fabsf(halfaxis_axis[ax]));
      secondpos = fminf(e1, secondpos);
      secondpos *= (float)mul;
    }
  } else if (cltype < 0) {
    // 胶囊端点最近于盒面：按第二方向找盒面/边上的最远可用点。
    if (clface != -1) {
      const int mul = (cltype == -3) ? 1 : -1;
      secondpos = 2.0f;
      const v3 tmp1 = v3_sub(pos, v3_scale(halfaxis, (float)mul));
      const float tmp1_axis[3] = {tmp1.x, tmp1.y, tmp1.z};
      for (int i = 0; i < 3; ++i) {
        if (i != clface) {
          const float ha_r = g5_primitive_safe_div((float)mul, halfaxis_axis[i]);
          float e1 = (size[i] - tmp1_axis[i]) * ha_r;
          if (0.0f < e1 && e1 < secondpos) secondpos = e1;
          e1 = (-size[i] - tmp1_axis[i]) * ha_r;
          if (0.0f < e1 && e1 < secondpos) secondpos = e1;
        }
      }
      secondpos *= (float)mul;
    }
  }

  // 第一接触：胶囊轴参数 bestsegmentpos 处的球端点。
  const v3 s1_pos_l = v3_add(pos, v3_scale(halfaxis, bestsegmentpos));
  const v3 s1_pos_g = v3_add(mul_mat3_vec(box_rot, s1_pos_l), box_pos);
  g5_primitive_pair pair;
  pair.contact[0] = g5_sphere_box(s1_pos_g, capsule_radius, box_pos, box_rot, box_size);

  if (secondpos > -3.0f) {
    const v3 s2_pos_l = v3_add(pos, v3_scale(halfaxis, secondpos + bestsegmentpos));
    const v3 s2_pos_g = v3_add(mul_mat3_vec(box_rot, s2_pos_l), box_pos);
    pair.contact[1] = g5_sphere_box(s2_pos_g, capsule_radius, box_pos, box_rot, box_size);
  } else {
    pair.contact[1].dist = G5_PRIMITIVE_MAXVAL;
    pair.contact[1].pos = v3_make(0.0f, 0.0f, 0.0f);
    pair.contact[1].normal = v3_make(0.0f, 0.0f, 0.0f);
  }
  return pair;
}
// G10@P5 G5：上游 `sphere_box_wrapper` / `capsule_box_wrapper` 的接触写入设备版。
// Adapted from frozen MJWarp `collision_primitive.py` (sphere_box_wrapper L1087-1157,
// capsule_box_wrapper L1160-1246), revision 71da24d. Apache-2.0.
//
// 只做「取几何字段 → 调 primitive 数值内核 → 逐接触 make_frame + write_contact」，
// 不复制上游 wrapper 参数列表（margin/gap/condim/friction/... 已在 `d_contact_params` 里）。

// 上游 `sphere_box_wrapper`：单接触，`write_contact(id_=0)`。
__device__ __forceinline__ void g5_sphere_box_wrapper(const d_geom& sphere, const d_geom& box,
                                                      const d_contact_params& params,
                                                      int naconmax, d_contact_out* out) {
  const g5_primitive_contact contact =
      g5_sphere_box(sphere.pos, sphere.size.x, box.pos, box.rot, box.size);
  float frame[9];
  make_frame(contact.normal, frame);
  g5_write_contact(params, naconmax, 0, contact.dist, contact.pos, frame, out);
}

// 上游 `capsule_box_wrapper`：两接触（`id_ = 0/1`）；无效第二接触的 `dist = MJ_MAXVAL`
// 由 `write_contact` 的 `detected` 门跳过，与上游计数语义一致。
__device__ __forceinline__ void g5_capsule_box_wrapper(const d_geom& capsule, const d_geom& box,
                                                       const d_contact_params& params,
                                                       int naconmax, d_contact_out* out) {
  // 上游 `axis = vec3(cap.rot[0, 2], cap.rot[1, 2], cap.rot[2, 2])`（行优先 mat33 的第三列）。
  const v3 axis = v3_make(capsule.rot[2], capsule.rot[5], capsule.rot[8]);
  const g5_primitive_pair pair =
      g5_capsule_box(capsule.pos, axis, capsule.size.x, capsule.size.y, box.pos, box.rot,
                     box.size);
  for (int i = 0; i < 2; ++i) {
    const g5_primitive_contact& contact = pair.contact[i];
    float frame[9];
    make_frame(contact.normal, frame);
    g5_write_contact(params, naconmax, i, contact.dist, contact.pos, frame, out);
  }
}
#pragma once
// G10@P5 G5：上游 MJWarp 多点 box-box 碰撞核心 `box_box` 设备版。
// Adapted from frozen MJWarp collision_primitive_core.py L563-591 (`_compute_rotmore`)
// and L595-1049 (`box_box`), revision 71da24d. Apache-2.0.
//
// 逐式 f32 转写：8 槽候选（dist 初值 MJ_MAXVAL=1e10）、3 面法线轴 + 9 边叉积轴、
// 面恢复与边恢复两条路径。mat3×mat3 用上游 `mat.h::mul` 的 fmaf 累加链
// （sum=0; sum=fmaf(...)，同 common.cu `relative_matrix` 口径）；mat3×vec3 用
// common.cu `mul_mat3_vec`；其余为直写 f32 表达式（不加代数化简、不用 double）。

// 上游 `types.MJ_MAXVAL`（mujoco.mjMAXVAL = 1e10）与 `types.MJ_MINVAL`（1e-15）。
#define G5_BB_MAXVAL 1.0e10f
#define G5_BB_MINVAL 1.0e-15f

struct g5_box_box_result {
    float dist[8];  // 未填充槽 = 1.0e10f (MJ_MAXVAL)
    v3 pos[8];
    v3 normal[8];
};

// 上游 `wp.vec3` 分量读（q=0/1/2）与分量写（用于 `points[n, q] += ...`）。
__device__ __forceinline__ float g5_bb_comp(v3 v, int q) {
    return q == 0 ? v.x : (q == 1 ? v.y : v.z);
}

__device__ __forceinline__ v3 g5_bb_set_comp(v3 v, int q, float x) {
    if (q == 0) { v.x = x; } else if (q == 1) { v.y = x; } else { v.z = x; }
    return v;
}

__device__ __forceinline__ void g5_bb_add_comp(v3* v, int q, float x) {
    if (q == 0) { v->x += x; } else if (q == 1) { v->y += x; } else { v->z += x; }
}

__device__ __forceinline__ v3 g5_bb_abs(v3 v) {
    return v3_make(fabsf(v.x), fabsf(v.y), fabsf(v.z));
}

// 上游 `wp.mat33` 的第 i 行（行优先）与 `wp.transpose(mat33)`。
__device__ __forceinline__ v3 g5_bb_row(const float* m, int i) {
    return v3_make(m[i * 3], m[i * 3 + 1], m[i * 3 + 2]);
}

__device__ __forceinline__ void g5_bb_transpose(const float* m, float* out) {
    for (int i = 0; i < 3; ++i) {
        for (int j = 0; j < 3; ++j) {
            out[i * 3 + j] = m[j * 3 + i];
        }
    }
}

// 上游 `mat.h::mul(mat33, mat33)`：`sum = 0; for k: sum = muladd(a[i][k], b[k][j], sum)`，
// `muladd(float)=fmaf`。显式 `__fmaf_rn` 固定 k 升序累加，避免依赖 `--fmad` 收缩顺序；
// rotmore 为 0/±1 置换阵时每个输出只有一个非零乘积，结果与上游逐位相同。
__device__ __forceinline__ void g5_bb_mat3_mul(const float* a, const float* b, float* out) {
    for (int i = 0; i < 3; ++i) {
        for (int j = 0; j < 3; ++j) {
            float sum = 0.0f;
            sum = __fmaf_rn(a[i * 3 + 0], b[0 * 3 + j], sum);
            sum = __fmaf_rn(a[i * 3 + 1], b[1 * 3 + j], sum);
            sum = __fmaf_rn(a[i * 3 + 2], b[2 * 3 + j], sum);
            out[i * 3 + j] = sum;
        }
    }
}

// 上游 `math.safe_div`：`x / where(y != 0.0, y, MJ_MINVAL)`。
__device__ __forceinline__ float g5_bb_safe_div(float x, float y) {
    return x / (y != 0.0f ? y : G5_BB_MINVAL);
}

// 上游 `_compute_rotmore(face_idx)`（L563-591）：6 个面各一个 0/±1 正交阵。
__device__ __forceinline__ void g5_bb_compute_rotmore(int face_idx, float* rotmore) {
    for (int i = 0; i < 9; ++i) rotmore[i] = 0.0f;
    if (face_idx == 0) {
        rotmore[2] = -1.0f; rotmore[4] = 1.0f; rotmore[6] = 1.0f;
    } else if (face_idx == 1) {
        rotmore[0] = 1.0f; rotmore[5] = -1.0f; rotmore[7] = 1.0f;
    } else if (face_idx == 2) {
        rotmore[0] = 1.0f; rotmore[4] = 1.0f; rotmore[8] = 1.0f;
    } else if (face_idx == 3) {
        rotmore[2] = 1.0f; rotmore[4] = 1.0f; rotmore[6] = -1.0f;
    } else if (face_idx == 4) {
        rotmore[0] = 1.0f; rotmore[5] = 1.0f; rotmore[7] = -1.0f;
    } else if (face_idx == 5) {
        rotmore[0] = -1.0f; rotmore[4] = 1.0f; rotmore[8] = -1.0f;
    }
}

// 上游 `box_box`：返回 8 个候选接触（未填充槽 dist=MJ_MAXVAL、pos/normal 全零）。
__device__ __forceinline__ g5_box_box_result g5_box_box(
    v3 pos1, const float* rot1, v3 size1,
    v3 pos2, const float* rot2, v3 size2, float margin) {
    g5_box_box_result result;
    for (int i = 0; i < 8; ++i) {
        result.dist[i] = G5_BB_MAXVAL;
        result.pos[i] = v3_make(0.0f, 0.0f, 0.0f);
        result.normal[i] = v3_make(0.0f, 0.0f, 0.0f);
    }

    // 盒间坐标变换（L630-640）。
    const v3 pos21 = transpose_mul_mat3_vec(rot1, v3_sub(pos2, pos1));
    const v3 pos12 = transpose_mul_mat3_vec(rot2, v3_sub(pos1, pos2));
    float rot1t[9], rot21[9], rot12[9], rot21abs[9], rot12abs[9];
    g5_bb_transpose(rot1, rot1t);
    g5_bb_mat3_mul(rot1t, rot2, rot21);
    g5_bb_transpose(rot21, rot12);
    for (int i = 0; i < 9; ++i) rot21abs[i] = fabsf(rot21[i]);
    g5_bb_transpose(rot21abs, rot12abs);
    const v3 plen2 = mul_mat3_vec(rot21abs, size2);
    const v3 plen1 = mul_mat3_vec(rot12abs, size1);

    const float s1[3] = {size1.x, size1.y, size1.z}, s2[3] = {size2.x, size2.y, size2.z};
    const float p21[3] = {pos21.x, pos21.y, pos21.z}, pl2[3] = {plen2.x, plen2.y, plen2.z};
    const float p12[3] = {pos12.x, pos12.y, pos12.z}, pl1[3] = {plen1.x, plen1.y, plen1.z};

    // 最大分离轴初值（L642-645）。
    const v3 s_sum_3 = v3_scale(v3_add(size1, size2), 3.0f);
    float separation = margin + s_sum_3.x + s_sum_3.y + s_sum_3.z;
    int axis_code = -1;

    // 第一段：盒面法线轴（L647-661）。
    for (int i = 0; i < 3; ++i) {
        const float c1 = -fabsf(p21[i]) + s1[i] + pl2[i];
        const float c2 = -fabsf(p12[i]) + s2[i] + pl1[i];
        if (c1 < -margin || c2 < -margin) return result;
        if (c1 < separation) {
            separation = c1;
            axis_code = i + 3 * (p21[i] < 0.0f ? 1 : 0) + 0;
        }
        if (c2 < separation) {
            separation = c2;
            axis_code = i + 3 * (p12[i] < 0.0f ? 1 : 0) + 6;
        }
    }

    v3 clnorm = v3_make(0.0f, 0.0f, 0.0f);
    bool inv = false;
    int cle1 = 0, cle2 = 0;

    // 第二段：盒边叉积轴（L668-717）。
    for (int i = 0; i < 3; ++i) {
        for (int j = 0; j < 3; ++j) {
            v3 cross_axis;
            if (i == 0) {
                cross_axis = v3_make(0.0f, -rot12[j * 3 + 2], rot12[j * 3 + 1]);
            } else if (i == 1) {
                cross_axis = v3_make(rot12[j * 3 + 2], 0.0f, -rot12[j * 3 + 0]);
            } else {
                cross_axis = v3_make(-rot12[j * 3 + 1], rot12[j * 3 + 0], 0.0f);
            }
            const float cross_length = v3_length(cross_axis);
            if (cross_length < G5_BB_MINVAL) continue;
            cross_axis = v3_make(cross_axis.x / cross_length, cross_axis.y / cross_length,
                                 cross_axis.z / cross_length);

            const float box_dist = v3_dot(pos21, cross_axis);
            float c3 = 0.0f;
            for (int k = 0; k < 3; ++k) {
                if (k != i) c3 += s1[k] * fabsf(g5_bb_comp(cross_axis, k));
                if (k != j) c3 += s2[k] * rot21abs[i * 3 + 3 - k - j] / cross_length;
            }
            c3 -= fabsf(box_dist);
            if (c3 < -margin) return result;

            if (c3 < separation * (1.0f - 1e-12f)) {
                separation = c3;
                cle1 = 0;
                cle2 = 0;
                for (int k = 0; k < 3; ++k) {
                    if (k != i &&
                        (((g5_bb_comp(cross_axis, k) > 0.0f) ? 1 : 0) ^
                         ((box_dist < 0.0f) ? 1 : 0))) {
                        cle1 += 1 << k;
                    }
                    if (k != j) {
                        const int t = (((rot21[i * 3 + 3 - k - j] > 0.0f) ? 1 : 0) ^
                                       ((box_dist < 0.0f) ? 1 : 0) ^
                                       ((((k - j + 3) % 3 == 1)) ? 1 : 0));
                        if (t) cle2 += 1 << k;
                    }
                }
                axis_code = 12 + i * 3 + j;
                clnorm = cross_axis;
                inv = box_dist < 0.0f;
            }
        }
    }
    if (axis_code == -1) return result;

    v3 points[8];
    float depth[8];
    for (int i = 0; i < 8; ++i) {
        points[i] = v3_make(0.0f, 0.0f, 0.0f);
        depth[i] = 0.0f;
    }

    int n = 0;
    float rw[9], hz = 0.0f;
    v3 pw = v3_make(0.0f, 0.0f, 0.0f), normal = v3_make(0.0f, 0.0f, 0.0f);

    if (axis_code < 12) {
        // 面-顶点路径（L728-837）。
        const int face_idx = axis_code % 6, box_idx = axis_code / 6;
        const bool box2 = box_idx != 0;
        float rotmore[9], r[9], rt[9], rwt[9];
        g5_bb_compute_rotmore(face_idx, rotmore);
        g5_bb_mat3_mul(rotmore, box2 ? rot12 : rot21, r);
        const v3 p = mul_mat3_vec(rotmore, box2 ? pos12 : pos21);
        const v3 s = box2 ? size1 : size2;
        const v3 ss = g5_bb_abs(mul_mat3_vec(rotmore, box2 ? size2 : size1));
        v3 lp = v3_make(p.x, p.y, p.z - ss.z);
        const float s_axis[3] = {s.x, s.y, s.z};
        g5_bb_transpose(r, rt);
        const float lx = ss.x, ly = ss.y;

        int clcorner = 0;
        for (int i = 0; i < 3; ++i) {
            if (r[6 + i] < 0.0f) clcorner += 1 << i;
        }
        for (int i = 0; i < 3; ++i) {
            const float corner_sign = (clcorner & (1 << i)) ? 1.0f : -1.0f;
            lp = v3_add(lp, v3_scale(v3_scale(g5_bb_row(rt, i), s_axis[i]), corner_sign));
        }

        v3 cn1 = v3_make(0.0f, 0.0f, 0.0f), cn2 = cn1;
        int dirs = 0;
        for (int i = 0; i < 3; ++i) {
            if (!(fabsf(r[6 + i]) < 0.5f)) continue;
            const float cn_sign = (clcorner & (1 << i)) ? -2.0f : 2.0f;
            const v3 cn = v3_scale(v3_scale(g5_bb_row(rt, i), s_axis[i]), cn_sign);
            if (dirs == 0) cn1 = cn; else cn2 = cn;
            ++dirs;
        }

        const int kk = dirs * dirs;
        n = 0;
        const v3 zero = v3_make(0.0f, 0.0f, 0.0f);

        // 盒边与盒面线段的交点（L772-792）。
        for (int i = 0; i < kk; ++i) {
            for (int q = 0; q < 2; ++q) {
                const v3 lav = v3_add(lp, i < 2 ? zero : (i == 2 ? cn1 : cn2));
                const v3 lbv = (i == 0 || i == 3) ? cn1 : cn2;
                const float lbv_q = g5_bb_comp(lbv, q);
                if (!(fabsf(lbv_q) > G5_BB_MINVAL)) continue;
                const float br = 1.0f / lbv_q;
                for (int jj = -1; jj <= 1; jj += 2) {
                    const float l = g5_bb_comp(ss, q) * (float)jj;
                    const float c1 = (l - g5_bb_comp(lav, q)) * br;
                    if (c1 < 0.0f || c1 > 1.0f) continue;
                    const float c2 = g5_bb_comp(lav, 1 - q) + g5_bb_comp(lbv, 1 - q) * c1;
                    if (fabsf(c2) > g5_bb_comp(ss, 1 - q)) continue;
                    points[n] = v3_add(lav, v3_scale(lbv, c1));
                    ++n;
                }
            }
        }

        // 两面相交时的网格点（L794-819）。
        if (dirs == 2) {
            const float ax = cn1.x, bx = cn2.x, ay = cn1.y, by = cn2.y;
            const float cdet = g5_bb_safe_div(1.0f, ax * by - bx * ay);
            for (int i = 0; i < 4; ++i) {
                const float llx = (i / 2) ? lx : -lx, lly = (i % 2) ? ly : -ly;
                const float x = llx - lp.x, y = lly - lp.y;
                const float u = (x * by - y * bx) * cdet, v = (y * ax - x * ay) * cdet;
                if (u > 0.0f && v > 0.0f && u < 1.0f && v < 1.0f) {
                    points[n] = v3_make(llx, lly, lp.z + u * cn1.z + v * cn2.z);
                    ++n;
                }
            }
        }

        for (int i = 0; i < (1 << dirs); ++i) {
            const v3 tmpv = v3_add(v3_add(lp, v3_scale(cn1, (float)(i & 1))),
                                   v3_scale(cn2, (i & 2) ? 1.0f : 0.0f));
            if (tmpv.x > -lx && tmpv.x < lx && tmpv.y > -ly && tmpv.y < ly) {
                points[n] = tmpv;
                ++n;
            }
        }

        // 折算深度并取位置中点（L821-833）。
        const int m = n;
        n = 0;
        for (int i = 0; i < m; ++i) {
            if (points[i].z > margin) continue;
            if (i != n) points[n] = points[i];
            depth[n] = points[n].z;
            points[n].z = points[n].z * 0.5f;
            ++n;
        }

        g5_bb_transpose(rotmore, rwt);
        g5_bb_mat3_mul(box2 ? rot2 : rot1, rwt, rw);
        pw = box2 ? pos2 : pos1;
        const float nsign = box2 ? -1.0f : 1.0f;
        normal = v3_make(nsign * rw[2], nsign * rw[5], nsign * rw[8]);
        hz = ss.z;
    } else {
        // 边-边路径（L840-1035）。
        const int edge1 = (axis_code - 12) / 3, edge2 = (axis_code - 12) % 3;
        int ax1 = 1 - (edge2 & 1), ax2 = 2 - (edge2 & 2);
        int pax1 = 1 - (edge1 & 1), pax2 = 2 - (edge1 & 2);
        if (rot21abs[edge1 * 3 + ax1] < rot21abs[edge1 * 3 + ax2]) {
            const int t = ax1; ax1 = ax2; ax2 = t;
        }
        if (rot12abs[edge2 * 3 + pax1] < rot12abs[edge2 * 3 + pax2]) {
            const int t = pax1; pax1 = pax2; pax2 = t;
        }

        float rotmore[9], r[9], rt[9], rwt[9], rw_scaled[9];
        g5_bb_compute_rotmore((cle1 & (1 << pax2)) ? pax2 : pax2 + 3, rotmore);
        v3 p = mul_mat3_vec(rotmore, pos21);
        const v3 rnorm = mul_mat3_vec(rotmore, clnorm);
        g5_bb_mat3_mul(rotmore, rot21, r);
        g5_bb_transpose(r, rt);
        const v3 sv = g5_bb_abs(transpose_mul_mat3_vec(rotmore, size1));
        const float s_axis[3] = {sv.x, sv.y, sv.z};
        const float lx = sv.x, ly = sv.y;
        hz = sv.z;
        p.z = p.z - hz;

        // 盒2最近面的 4 个候选点（L871-888）。
        const v3 rtax1 = g5_bb_row(rt, ax1), rtax2 = g5_bb_row(rt, ax2);
        const v3 rt_edge = g5_bb_row(rt, edge2);
        const float sign_ax1 = (cle2 & (1 << ax1)) ? 1.0f : -1.0f;
        const float sign_ax2 = (cle2 & (1 << ax2)) ? 1.0f : -1.0f;
        const v3 term_ax1 = v3_scale(v3_scale(rtax1, s2[ax1]), sign_ax1);
        const v3 term_ax2 = v3_scale(v3_scale(rtax2, s2[ax2]), sign_ax2);
        const v3 term_ax1n = v3_scale(v3_scale(rtax1, s2[ax1]), -sign_ax1);
        const v3 edge_term = v3_scale(rt_edge, s2[edge2]);
        points[0] = v3_add(v3_add(p, term_ax1), term_ax2);
        points[1] = v3_sub(points[0], edge_term);
        points[0] = v3_add(points[0], edge_term);
        points[2] = v3_add(v3_add(p, term_ax1n), term_ax2);
        points[3] = v3_sub(points[2], edge_term);
        points[2] = v3_add(points[2], edge_term);
        n = 4;

        const v3 axi_lp = points[0];
        const v3 axi_cn1 = v3_sub(points[1], points[0]);
        const v3 axi_cn2 = v3_sub(points[2], points[0]);
        if (fabsf(rnorm.z) < G5_BB_MINVAL) return result;
        const float inv_sign = inv ? -1.0f : 1.0f;
        const float innorm = inv_sign / rnorm.z;

        v3 pu[4];
        for (int i = 0; i < 4; ++i) {
            pu[i] = points[i];
            const float c_scl = points[i].z * inv_sign * innorm;
            points[i] = v3_sub(points[i], v3_scale(rnorm, c_scl));
        }
        const v3 pts_lp = points[0];
        const v3 pts_cn1 = v3_sub(points[1], points[0]);
        const v3 pts_cn2 = v3_sub(points[2], points[0]);
        n = 0;

        // 盒1边的投影线段与盒2投影面求交（L916-947）。
        for (int i = 0; i < 4; ++i) {
            for (int q = 0; q < 2; ++q) {
                const float la = g5_bb_comp(pts_lp, q) +
                                 (i < 2 ? 0.0f
                                        : (i == 2 ? g5_bb_comp(pts_cn1, q)
                                                  : g5_bb_comp(pts_cn2, q)));
                const float lb = (i == 0 || i == 3) ? g5_bb_comp(pts_cn1, q)
                                                    : g5_bb_comp(pts_cn2, q);
                const float lc = g5_bb_comp(pts_lp, 1 - q) +
                                 (i < 2 ? 0.0f
                                        : (i == 2 ? g5_bb_comp(pts_cn1, 1 - q)
                                                  : g5_bb_comp(pts_cn2, 1 - q)));
                const float ld = (i == 0 || i == 3) ? g5_bb_comp(pts_cn1, 1 - q)
                                                    : g5_bb_comp(pts_cn2, 1 - q);
                const v3 lua = v3_add(axi_lp, i < 2 ? v3_make(0.0f, 0.0f, 0.0f)
                                                    : (i == 2 ? axi_cn1 : axi_cn2));
                const v3 lub = (i == 0 || i == 3) ? axi_cn1 : axi_cn2;
                if (!(fabsf(lb) > G5_BB_MINVAL)) continue;
                const float br = 1.0f / lb;
                for (int jj = -1; jj <= 1; jj += 2) {
                    if (n == 8) break;
                    const float l = g5_bb_comp(sv, q) * (float)jj;
                    const float c1 = (l - la) * br;
                    if (c1 < 0.0f || c1 > 1.0f) continue;
                    const float c2 = lc + ld * c1;
                    if (fabsf(c2) > g5_bb_comp(sv, 1 - q)) continue;
                    if ((lua.z + lub.z * c1) * innorm > margin) continue;
                    points[n] = v3_add(v3_scale(lua, 0.5f),
                                       v3_scale(v3_scale(lub, c1), 0.5f));
                    g5_bb_add_comp(&points[n], q, 0.5f * l);
                    g5_bb_add_comp(&points[n], 1 - q, 0.5f * c2);
                    depth[n] = points[n].z * innorm * 2.0f;
                    ++n;
                }
            }
        }

        // 投影面角点/网格点（L950-990）。
        const int nl = n;
        const float ax = pts_cn1.x, bx = pts_cn2.x;
        const float ay = pts_cn1.y, by = pts_cn2.y;
        const float cdet = g5_bb_safe_div(1.0f, ax * by - bx * ay);
        for (int i = 0; i < 4; ++i) {
            if (n == 8) break;
            const float llx = (i / 2) ? lx : -lx, lly = (i % 2) ? ly : -ly;
            const float x = llx - pts_lp.x, y = lly - pts_lp.y;
            float u = (x * by - y * bx) * cdet, v = (y * ax - x * ay) * cdet;
            if (nl == 0) {
                if ((u < 0.0f || u > 1.0f) && (v < 0.0f || v > 1.0f)) continue;
            } else if (u < 0.0f || v < 0.0f || u > 1.0f || v > 1.0f) continue;
            u = fminf(fmaxf(0.0f, u), 1.0f);
            v = fminf(fmaxf(0.0f, v), 1.0f);
            const float w = 1.0f - u - v;
            const v3 vtmp = v3_add(v3_add(v3_scale(pu[0], w), v3_scale(pu[1], u)),
                                   v3_scale(pu[2], v));
            points[n] = v3_make(llx, lly, 0.0f);
            const v3 vtmp2 = v3_sub(points[n], vtmp);
            const float tc1 = v3_dot(vtmp2, vtmp2);
            if (vtmp.z > 0.0f && tc1 > margin * margin) continue;
            points[n] = v3_scale(v3_add(points[n], vtmp), 0.5f);
            depth[n] = sqrtf(tc1) * (vtmp.z < 0.0f ? -1.0f : 1.0f);
            ++n;
        }

        // 盒2投影面的顶点（L993-1029）。
        const int nf = n;
        for (int i = 0; i < 4; ++i) {
            if (n >= 8) break;
            const float x = g5_bb_comp(pu[i], 0), y = g5_bb_comp(pu[i], 1);
            if (nl == 0 && nf != 0) {
                if ((x < -lx || x > lx) && (y < -ly || y > ly)) continue;
            } else if (x < -lx || x > lx || y < -ly || y > ly) continue;

            float c1 = 0.0f;
            for (int j = 0; j < 2; ++j) {
                const float puj = g5_bb_comp(pu[i], j);
                if (puj < -s_axis[j]) c1 += (puj + s_axis[j]) * (puj + s_axis[j]);
                else if (puj > s_axis[j]) c1 += (puj - s_axis[j]) * (puj - s_axis[j]);
            }
            const float pu2 = g5_bb_comp(pu[i], 2);
            c1 += pu2 * innorm * pu2 * innorm;
            if (pu2 > 0.0f && c1 > margin * margin) continue;

            v3 tmp_p = v3_make(x, y, 0.0f);
            for (int j = 0; j < 2; ++j) {
                const float puj = g5_bb_comp(pu[i], j);
                if (puj < -s_axis[j]) tmp_p = g5_bb_set_comp(tmp_p, j, -s_axis[j] * 0.5f);
                else if (puj > s_axis[j]) tmp_p = g5_bb_set_comp(tmp_p, j, s_axis[j] * 0.5f);
            }
            tmp_p = v3_add(tmp_p, pu[i]);
            points[n] = v3_scale(tmp_p, 0.5f);
            depth[n] = sqrtf(c1) * (pu2 < 0.0f ? -1.0f : 1.0f);
            ++n;
        }

        // 接触面基（L1031-1034）：`(scalar * rw) @ rnorm`，先缩放矩阵再乘向量。
        g5_bb_transpose(rotmore, rwt);
        g5_bb_mat3_mul(rot1, rwt, rw);
        pw = pos1;
        const float nsign = inv ? -1.0f : 1.0f;
        for (int i = 0; i < 9; ++i) rw_scaled[i] = nsign * rw[i];
        normal = mul_mat3_vec(rw_scaled, rnorm);
    }

    // 输出（L1036-1046）：先还原 z，再变换到世界系。
    for (int i = 0; i < n; ++i) {
        points[i].z = points[i].z + hz;
        result.dist[i] = depth[i];
        result.pos[i] = v3_add(mul_mat3_vec(rw, points[i]), pw);
        result.normal[i] = normal;
    }
    return result;
}
#pragma once
// G10@P5 G5：上游 `box_box_wrapper`（collision_primitive.py L1250-1352）设备版。
// Adapted from frozen MJWarp collision_primitive.py, revision 71da24d. Apache-2.0.
//
// 调用方保证 contact.cu（`g5_write_contact` / `make_frame`）已先包含；本文件只做
// 「取几何字段 → g5_box_box → 8 槽逐 i make_frame + write_contact」。未填充槽的
// dist=MJ_MAXVAL 由 `g5_write_contact` 的 detected 门过滤，与上游计数语义一致。

__device__ __forceinline__ void g5_box_box_wrapper(
    const d_geom& box1, const d_geom& box2, const d_contact_params& params,
    int naconmax, d_contact_out* out) {
    const g5_box_box_result result = g5_box_box(
        box1.pos, box1.rot, box1.size, box2.pos, box2.rot, box2.size, params.margin);
    for (int i = 0; i < 8; ++i) {
        float frame[9];
        make_frame(result.normal[i], frame);
        g5_write_contact(params, naconmax, i, result.dist[i], result.pos[i], frame, out);
    }
}
// G10@P5 B1 plane 族 primitive 设备实现（拆分：common）
// 语义来源 pinned collision_primitive.py rev 71da24d9；设备保持 GPU fma.rn 融合语义；0 新 ABI 字段。

// G10@P5 B1：plane 族 primitive 设备实现（pinned collision_primitive.py rev 71da24d9；0 新 ABI 字段）。
//
// 语义来源（pinned，只读）：mujoco_warp 3.15 `collision_primitive.py`
//   - `plane_convex(plane_normal, plane_pos, convex)`：L52-278（共享核心）
//   - `plane_sphere_wrapper`：L281-355
//   - `plane_box_wrapper`：L761-837
//   - `plane_convex_wrapper`（mesh 走此 wrapper）：L838-914
// revision 71da24d956378a87a703b6e1442b13aec0c4ac29；设备侧保持 pinned GPU fma.rn 融合语义。
// 接触写入/结构字段复用既有 `primitive_wrapper.cu` 的 `g5_write_contact`/`make_frame`，
// 本文件不新增接触写入器、不改 ABI（`CollisionArgs`/`CollisionContactOut` 零新增字段）。
//


// 上游 `types.MJ_MAXVAL`（未填接触位置用）。
#define G5_PLANE_MAXVAL 1.0e10f
// 上游 `plane_convex` 的 `_HUGE_VAL`。
#define G5_PLANE_HUGE_VAL 1.0e6f
// `plane_convex` 的阈值（pinned L94 `max_support - 1e-3`）。
#define G5_PLANE_CONVEX_THRESHOLD 1.0e-3f

// 单条 plane 族接触候选（对应上游 wrapper 内 `(dist, pos, normal)`）。
struct g5_plane_contact {
  float dist;
  v3 pos;
  v3 normal;
};


// (n,b,c) 行优先 3x3 帧（plane_capsule 使用；与上游 wp.mat33(n,b,c) 行序一致）。
//   contact 2 = plane_sphere(n, plane_pos, capsule_pos-segment, radius)。
// ---------------------------------------------------------------------------------------------

// 把 (n, b, c) 行优先 3x3 帧写成设备侧 float[9]（与上游 `wp.mat33(n,b,c)` 行序一致）。
__device__ __forceinline__ void g5_plane_frame(v3 n, v3 b, v3 c, float frame[9]) {
  frame[0] = n.x; frame[1] = n.y; frame[2] = n.z;
  frame[3] = b.x; frame[4] = b.y; frame[5] = b.z;
  frame[6] = c.x; frame[7] = c.y; frame[8] = c.z;
}


// G10@P5 B1 plane_convex 设备实现（pinned collision_primitive.py L52-278 平移）。
// 唯一入口：g5_plane_convex（穷举 + graph hillclimb + 4 点去重接触收集）。

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

// G10@P5 B1 plane 族数值核心（shapes：sphere / box / capsule；ellipsoid / cylinder 下一块）。
// 语义来源 pinned collision_primitive_core.py；复用既有 g5_write_contact/make_frame（不改 ABI）。

// 上游 `collision_primitive_core.plane_sphere`：
//   dist = dot(sphere_pos - plane_pos, plane_normal) - radius
//   pos  = sphere_pos - plane_normal * (radius + 0.5 * dist)
__device__ __forceinline__ void g5_plane_sphere(v3 plane_normal, v3 plane_pos, v3 sphere_pos,
                                                float sphere_radius, float* dist, v3* pos) {
  *dist = v3_dot(v3_sub(sphere_pos, plane_pos), plane_normal) - sphere_radius;
  *pos = v3_sub(sphere_pos, v3_scale(plane_normal, sphere_radius + 0.5f * (*dist)));
}



// 上游 `plane_box_wrapper`（pinned L761-837）几何核心：8 角点逐点距离与位置。
// 每角点：corner = box_pos + box_rot * (size * sign)；dist = dot(corner - plane_pos, normal)；
// pos = corner - 0.5 * dist * normal（pinned plane_convex L269-271 同式）。
__device__ __forceinline__ int g5_plane_box(v3 plane_normal, v3 plane_pos, v3 box_pos,
                                            const float* box_rot, v3 box_size,
                                            g5_plane_contact out[8]) {
  // pinned `plane_box`（core L366-390）：center_dist = dot(box_pos - plane_pos, n)；
  // ldist = dot(n, rot@corner_local)；cdist = center_dist + ldist；
  // pos = rot@corner_local + box_pos - (0.5*plane_normal)*cdist（左结合，逐式对齐 pinned）。
  const float center_dist = v3_dot(v3_sub(box_pos, plane_pos), plane_normal);
  int count = 0;
  for (int i = 0; i < 8; ++i) {
    const float sx = (i & 1) ? 1.0f : -1.0f;
    const float sy = (i & 2) ? 1.0f : -1.0f;
    const float sz = (i & 4) ? 1.0f : -1.0f;
    const v3 local = v3_make(sx * box_size.x, sy * box_size.y, sz * box_size.z);
    const v3 rotated = mul_mat3_vec(box_rot, local);
    const float ldist = v3_dot(plane_normal, rotated);
    const float cdist = center_dist + ldist;
    out[count].dist = cdist;
    out[count].pos = v3_sub(v3_add(rotated, box_pos),
                            v3_scale(v3_scale(plane_normal, 0.5f), cdist));
    out[count].normal = plane_normal;
    count++;
  }
  return count;
}




__device__ __forceinline__ int g5_plane_capsule(v3 plane_normal, v3 plane_pos, v3 capsule_pos,
                                                v3 capsule_axis, float capsule_radius,
                                                float capsule_half_length,
                                                g5_plane_contact out[2], float frame[9]) {
  const v3 n = plane_normal;
  const v3 axis = capsule_axis;

  // b = normalize(axis - n*dot(n,axis))，零保护沿用上游 `normalize_with_norm`（L287）。
  v3 b = v3_make(0.0f, 0.0f, 0.0f);
  float b_norm = 0.0f;
  const v3 perpendicular = v3_sub(axis, v3_scale(n, v3_dot(n, axis)));
  const float length = sqrtf(v3_dot(perpendicular, perpendicular));
  b_norm = length;
  if (length == 0.0f) {
    b = perpendicular;
  } else {
    b = v3_make(perpendicular.x / length, perpendicular.y / length, perpendicular.z / length);
  }
  if (b_norm < 0.5f) {
    if (-0.5f < n.y && n.y < 0.5f) {
      b = v3_make(0.0f, 1.0f, 0.0f);
    } else {
      b = v3_make(0.0f, 0.0f, 1.0f);
    }
  }
  const v3 c = v3_cross(n, b);
  g5_plane_frame(n, b, c, frame);

  const v3 segment = v3_scale(axis, capsule_half_length);
  g5_plane_sphere(n, plane_pos, v3_add(capsule_pos, segment), capsule_radius, &out[0].dist, &out[0].pos);
  out[0].normal = n;
  g5_plane_sphere(n, plane_pos, v3_sub(capsule_pos, segment), capsule_radius, &out[1].dist, &out[1].pos);
  out[1].normal = n;
  return 2;
}


// ---------------------------------------------------------------------------------------------
// plane_ellipsoid / plane_cylinder 数值核心（pinned collision_primitive_core.py L311-339 / L466-559）
// ---------------------------------------------------------------------------------------------

// 上游 `plane_ellipsoid`（pinned L312-339）：
//   sphere_support = -normalize(cw_mul(transpose(rot) @ n, size))
//   pos = ellipsoid_pos + rot @ cw_mul(sphere_support, size)
//   dist = dot(n, pos - plane_pos); pos -= n * dist * 0.5
// 退化保护（本片追加，非上游路径）：size 或局部法线缩放到 0 时返回 0 接触，不产生 NaN。
__device__ __forceinline__ int g5_plane_ellipsoid(v3 plane_normal, v3 plane_pos, v3 ellipsoid_pos,
                                                  const float* ellipsoid_rot, v3 ellipsoid_size,
                                                  g5_plane_contact out[1]) {
  const v3 local_n = transpose_mul_mat3_vec(ellipsoid_rot, plane_normal);
  const v3 scaled = v3_make(local_n.x * ellipsoid_size.x, local_n.y * ellipsoid_size.y,
                            local_n.z * ellipsoid_size.z);
  const float length = sqrtf(v3_dot(scaled, scaled));
  if (!(length > 0.0f) || !isfinite(length)) {
    return 0;
  }
  const v3 normalized = v3_make(scaled.x / length, scaled.y / length, scaled.z / length);
  const v3 sphere_support = v3_make(-normalized.x, -normalized.y, -normalized.z);
  const v3 local = v3_make(sphere_support.x * ellipsoid_size.x,
                           sphere_support.y * ellipsoid_size.y,
                           sphere_support.z * ellipsoid_size.z);
  v3 pos = v3_add(ellipsoid_pos, mul_mat3_vec(ellipsoid_rot, local));
  const float dist = v3_dot(plane_normal, v3_sub(pos, plane_pos));
  pos = v3_sub(pos, v3_scale(v3_scale(plane_normal, dist), 0.5f));
  out[0].dist = dist;
  out[0].pos = pos;
  out[0].normal = plane_normal;
  return 1;
}

// 上游 `plane_cylinder`（pinned L466-559）：4 条候选（两端面 + 侧面三角两点）。
// 退化保护：`len_sqr >= 1e-12` 走 normalize/safe_div，否则回退 x 轴 * radius（pinned L513-517）。
__device__ __forceinline__ int g5_plane_cylinder(v3 plane_normal, v3 plane_pos, v3 cylinder_center,
                                                 v3 cylinder_axis, float cylinder_radius,
                                                 float cylinder_half_height,
                                                 g5_plane_contact out[4]) {
  const v3 n = plane_normal;
  v3 axis = cylinder_axis;
  float prjaxis = v3_dot(n, axis);
  if (prjaxis > 0.0f) {
    axis = v3_scale(axis, -1.0f);
    prjaxis = -prjaxis;
  }
  const float dist0 = v3_dot(v3_sub(cylinder_center, plane_pos), n);
  v3 vec = v3_sub(v3_scale(axis, prjaxis), n);
  const float len_sqr = v3_dot(vec, vec);
  if (len_sqr >= 1.0e-12f) {
    const float len = sqrtf(len_sqr);
    const float denominator = len != 0.0f ? len : 1.0e-15f;
    vec = v3_scale(vec, cylinder_radius / denominator);
  } else {
    vec = v3_make(cylinder_radius, 0.0f, 0.0f);
  }
  const float prjvec = v3_dot(vec, n);
  axis = v3_scale(axis, cylinder_half_height);
  prjaxis = prjaxis * cylinder_half_height;

  // 接触 1/2：两端面（pinned L526-538）。
  const float dist1 = dist0 + prjaxis + prjvec;
  out[0].dist = dist1;
  out[0].pos = v3_sub(v3_add(v3_add(cylinder_center, vec), axis), v3_scale(n, dist1 * 0.5f));
  out[0].normal = n;
  const float dist2 = dist0 - prjaxis + prjvec;
  out[1].dist = dist2;
  out[1].pos = v3_sub(v3_sub(v3_add(cylinder_center, vec), axis), v3_scale(n, dist2 * 0.5f));
  out[1].normal = n;

  // 接触 3/4：侧面近平面三角两点（pinned L540-557）。
  const float prjvec1 = -prjvec * 0.5f;
  const float dist3 = dist0 + prjaxis + prjvec1;
  v3 vec1 = v3_cross(vec, axis);
  const float vec1_len = sqrtf(v3_dot(vec1, vec1));
  const float vec1_denominator = vec1_len != 0.0f ? vec1_len : 1.0e-15f;
  vec1 = v3_make(vec1.x / vec1_denominator, vec1.y / vec1_denominator, vec1.z / vec1_denominator);
  vec1 = v3_scale(vec1, cylinder_radius * sqrtf(3.0f) * 0.5f);
  const v3 base = v3_sub(v3_add(cylinder_center, axis), v3_scale(vec, 0.5f));
  out[2].dist = dist3;
  out[2].pos = v3_sub(v3_add(base, vec1), v3_scale(n, dist3 * 0.5f));
  out[2].normal = n;
  out[3].dist = dist3;
  out[3].pos = v3_sub(v3_sub(base, vec1), v3_scale(n, dist3 * 0.5f));
  out[3].normal = n;
  return 4;
}

// G10@P5 B1：plane 族 6 个 wrapper 的接触写入门（设备版）。
// 语义来源 pinned `collision_primitive.py`：plane_sphere_wrapper L281、plane_capsule_wrapper L600、
// plane_ellipsoid_wrapper L686、plane_cylinder_wrapper L1000、plane_box_wrapper L761、
// plane_convex_wrapper L838-914；revision 71da24d。设备保持 pinned GPU fma.rn 融合语义。
//
// 写入约定与既有 `primitive_wrapper.cu` 完全一致（不新增写入器、不改 ABI）：
//   `g5_write_contact(params, naconmax, id_, dist, pos, frame, out)`；无效接触的 `dist = MJ_MAXVAL`
//   由 `g5_write_contact` 的 `detected` 门跳过（与上游计数语义一致）。`make_frame` 参数顺序为
//   法线在前（`make_frame(normal, frame)`），与既有 primitive 片相同。
//
// 入口复用：product.cu 的 `mjwarp_collision_primitive` 按 type pair 调用下面 wrapper；
// 本文件不新增 `extern "C"` 入口、不新增 ABI 字段。

// plane 族 wrapper 需要的 mesh 访问（product 参数里已有：mesh_vert / mesh_graph 指针）。
struct g5_plane_mesh_view {
  const float* vert;
  const int* graph;
};

// 上游 `plane_sphere_wrapper`（pinned L281-355）：单接触，id_=0。
__device__ __forceinline__ void g5_plane_sphere_wrapper(const d_geom& plane, const d_geom& sphere,
                                                        const d_contact_params& params,
                                                        int naconmax, d_contact_out* out) {
  const v3 normal = v3_make(plane.rot[2], plane.rot[5], plane.rot[8]);
  g5_plane_contact contact;
  g5_plane_sphere(normal, plane.pos, sphere.pos, sphere.size.x, &contact.dist, &contact.pos);
  contact.normal = normal;
  float frame[9];
  make_frame(contact.normal, frame);
  g5_write_contact(params, naconmax, 0, contact.dist, contact.pos, frame, out);
}

// 上游 `plane_capsule_wrapper`（pinned L600-685）：两接触 id_=0/1，共享 (n,b,c) 帧。
__device__ __forceinline__ void g5_plane_capsule_wrapper(const d_geom& plane, const d_geom& capsule,
                                                         const d_contact_params& params,
                                                         int naconmax, d_contact_out* out) {
  const v3 normal = v3_make(plane.rot[2], plane.rot[5], plane.rot[8]);
  const v3 axis = v3_make(capsule.rot[2], capsule.rot[5], capsule.rot[8]);
  g5_plane_contact contacts[2];
  float plane_frame[9];
  g5_plane_capsule(normal, plane.pos, capsule.pos, axis, capsule.size.x, capsule.size.y, contacts,
                   plane_frame);
  for (int i = 0; i < 2; ++i) {
    g5_write_contact(params, naconmax, i, contacts[i].dist, contacts[i].pos, plane_frame, out);
  }
}

// 上游 `plane_ellipsoid_wrapper`（pinned L686-760）：单接触 id_=0，frame = make_frame(normal)。
__device__ __forceinline__ void g5_plane_ellipsoid_wrapper(const d_geom& plane,
                                                           const d_geom& ellipsoid,
                                                           const d_contact_params& params,
                                                           int naconmax, d_contact_out* out) {
  const v3 normal = v3_make(plane.rot[2], plane.rot[5], plane.rot[8]);
  g5_plane_contact contacts[1];
  g5_plane_ellipsoid(normal, plane.pos, ellipsoid.pos, ellipsoid.rot, ellipsoid.size, contacts);
  float frame[9];
  make_frame(normal, frame);
  g5_write_contact(params, naconmax, 0, contacts[0].dist, contacts[0].pos, frame, out);
}

// 上游 `plane_cylinder_wrapper`（pinned L1000-1086）：4 条候选 id_=0..3，frame = make_frame(normal)；
// 无效候选的 MJ_MAXVAL 由 write_contact detected 门跳过。
__device__ __forceinline__ void g5_plane_cylinder_wrapper(const d_geom& plane,
                                                          const d_geom& cylinder,
                                                          const d_contact_params& params,
                                                          int naconmax, d_contact_out* out) {
  const v3 normal = v3_make(plane.rot[2], plane.rot[5], plane.rot[8]);
  const v3 axis = v3_make(cylinder.rot[2], cylinder.rot[5], cylinder.rot[8]);
  g5_plane_contact contacts[4];
  g5_plane_cylinder(normal, plane.pos, cylinder.pos, axis, cylinder.size.x, cylinder.size.y,
                    contacts);
  float frame[9];
  make_frame(normal, frame);
  for (int i = 0; i < 4; ++i) {
    g5_write_contact(params, naconmax, i, contacts[i].dist, contacts[i].pos, frame, out);
  }
}

// 上游 `plane_box_wrapper`（pinned L761-837）：8 角点候选 id_=0..7，frame = make_frame(normal)。
__device__ __forceinline__ void g5_plane_box_wrapper(const d_geom& plane, const d_geom& box,
                                                     const d_contact_params& params,
                                                     int naconmax, d_contact_out* out) {
  const v3 normal = v3_make(plane.rot[2], plane.rot[5], plane.rot[8]);
  g5_plane_contact contacts[8];
  const int count = g5_plane_box(normal, plane.pos, box.pos, box.rot, box.size, contacts);
  float frame[9];
  make_frame(normal, frame);
  for (int i = 0; i < count; ++i) {
    g5_write_contact(params, naconmax, i, contacts[i].dist, contacts[i].pos, frame, out);
  }
}

// 上游 `plane_convex_wrapper`（pinned L838-914）：mesh 走 `plane_convex` 核心，≤4 接触 id_=0..3，
// frame = make_frame(plane normal)。
__device__ __forceinline__ void g5_plane_convex_wrapper(const d_geom& plane, const d_geom& convex,
                                                        const g5_plane_mesh_view& mesh,
                                                        const d_contact_params& params, int naconmax,
                                                        d_contact_out* out) {
  const v3 normal = v3_make(plane.rot[2], plane.rot[5], plane.rot[8]);
  g5_plane_contact contacts[4];
  const int count = g5_plane_convex(normal, plane.pos, convex, mesh.vert, mesh.graph, contacts);
  float frame[9];
  make_frame(normal, frame);
  for (int i = 0; i < count; ++i) {
    g5_write_contact(params, naconmax, i, contacts[i].dist, contacts[i].pos, frame, out);
  }
}

// G10@P5 B2 sphere/capsule 族 primitive 数值核心（sphere-sphere / sphere-capsule /
// capsule-capsule / sphere-cylinder）。
// Adapted from frozen MJWarp `collision_primitive_core.py`
// （`sphere_sphere` L58-87、`sphere_capsule` L90-125、`capsule_capsule` L128-255、
// `sphere_cylinder` L393-462），revision 71da24d956378a87a703b6e1442b13aec0c4ac29。Apache-2.0。
//
// 逐表达式按 pinned 顺序转写（含 zero-distance fallback、平行/退化分支、端面+侧面+角点分支）；
// 上层 wrapper 与接触写入见 `primitive_sphere_capsule_wrapper.cu`。0 新 ABI 字段。

// 上游 `wp.inf`（`capsule_capsule` 无效接触距离；`write_contact` 的 `detected` 门会跳过）。
#define G5_SC_INF __int_as_float(0x7f800000)

// 上游 `collision_primitive_core.sphere_sphere`：
//   dir = pos2 - pos1；dist = |dir|；dist == 0 时 n = (1,0,0)，否则 n = dir / dist；
//   dist -= (radius1 + radius2)；pos = pos1 + n * (radius1 + 0.5 * dist)。
__device__ __forceinline__ g5_primitive_contact g5_sphere_sphere(v3 pos1, float radius1, v3 pos2,
                                                                 float radius2) {
  const v3 dir = v3_sub(pos2, pos1);
  float dist = v3_length(dir);
  v3 n;
  if (dist == 0.0f) {
    n = v3_make(1.0f, 0.0f, 0.0f);
  } else {
    // pinned `dir / dist`：逐分量除法（不是乘倒数）。
    n = v3_make(dir.x / dist, dir.y / dist, dir.z / dist);
  }
  dist = dist - (radius1 + radius2);
  const v3 pos = v3_add(pos1, v3_scale(n, radius1 + 0.5f * dist));
  g5_primitive_contact contact;
  contact.dist = dist;
  contact.pos = pos;
  contact.normal = n;
  return contact;
}

// 上游 `collision_primitive_core.sphere_capsule`：把球心投影到胶囊中心线并 clamp，
// 再对最近点调 `sphere_sphere`。
__device__ __forceinline__ g5_primitive_contact g5_sphere_capsule(v3 sphere_pos, float sphere_radius,
                                                                 v3 capsule_pos, v3 capsule_axis,
                                                                 float capsule_radius,
                                                                 float capsule_half_length) {
  const v3 offset = v3_sub(sphere_pos, capsule_pos);
  // 上游 `wp.clamp(dot(axis, offset), -half_length, half_length)`。
  const float projection = fminf(fmaxf(v3_dot(capsule_axis, offset), -capsule_half_length),
                                 capsule_half_length);
  const v3 pt = v3_add(capsule_pos, v3_scale(capsule_axis, projection));
  return g5_sphere_sphere(sphere_pos, sphere_radius, pt, capsule_radius);
}

// 上游 `collision_primitive_core.capsule_capsule`：非平行轴取 1 个接触（含 3 处分段
// clamp 修正），平行轴最多取 2 个端点接触；`margin` 过滤（`dist <= margin` 才写）。
// 返回两个接触槽；无效槽 `dist = wp.inf`（由 `write_contact` 跳过）。
__device__ __forceinline__ void g5_capsule_capsule_core(
    v3 cap1_pos, v3 cap1_axis, float cap1_radius, float cap1_half_length, v3 cap2_pos,
    v3 cap2_axis, float cap2_radius, float cap2_half_length, float margin,
    g5_primitive_pair* pair) {
  // 上游 `contact_dist = vec2(inf, inf)`、`contact_pos/contact_normal = mat23f()`（全零）。
  pair->contact[0].dist = G5_SC_INF;
  pair->contact[0].pos = v3_make(0.0f, 0.0f, 0.0f);
  pair->contact[0].normal = v3_make(0.0f, 0.0f, 0.0f);
  pair->contact[1] = pair->contact[0];

  // 缩放轴与中心差。
  const v3 axis1 = v3_scale(cap1_axis, cap1_half_length);
  const v3 axis2 = v3_scale(cap2_axis, cap2_half_length);
  const v3 dif = v3_sub(cap1_pos, cap2_pos);

  // 矩阵系数与行列式。
  const float ma = v3_dot(axis1, axis1);
  const float mb = -v3_dot(axis1, axis2);
  const float mc = v3_dot(axis2, axis2);
  const float u = -v3_dot(axis1, dif);
  const float v = v3_dot(axis2, dif);
  const float det = ma * mc - mb * mb;

  if (fabsf(det) >= G5_PRIMITIVE_MINVAL) {
    // 非平行轴：1 个接触。
    const float inv_det = 1.0f / det;
    float x1 = (mc * u - mb * v) * inv_det;
    float x2 = (ma * v - mb * u) * inv_det;

    if (x1 > 1.0f) {
      x1 = 1.0f;
      x2 = (v - mb) / mc;
    } else if (x1 < -1.0f) {
      x1 = -1.0f;
      x2 = (v + mb) / mc;
    }

    if (x2 > 1.0f) {
      x2 = 1.0f;
      x1 = fminf(fmaxf((u - mb) / ma, -1.0f), 1.0f);
    } else if (x2 < -1.0f) {
      x2 = -1.0f;
      x1 = fminf(fmaxf((u + mb) / ma, -1.0f), 1.0f);
    }

    const v3 vec1 = v3_add(cap1_pos, v3_scale(axis1, x1));
    const v3 vec2 = v3_add(cap2_pos, v3_scale(axis2, x2));
    const g5_primitive_contact contact = g5_sphere_sphere(vec1, cap1_radius, vec2, cap2_radius);
    if (contact.dist <= margin) {
      pair->contact[0] = contact;
    }
  } else {
    // 平行轴：逐对测试 4 个端点，保留通过 margin 检查的前 2 个。
    int contact_count = 0;

    // x1 = 1：胶囊 1 正端。
    {
      const v3 vec1 = v3_add(cap1_pos, axis1);
      const float x2 = fminf(fmaxf((v - mb) / mc, -1.0f), 1.0f);
      const v3 vec2 = v3_add(cap2_pos, v3_scale(axis2, x2));
      const g5_primitive_contact contact = g5_sphere_sphere(vec1, cap1_radius, vec2, cap2_radius);
      if (contact.dist <= margin) {
        pair->contact[contact_count] = contact;
        contact_count += 1;
      }
    }

    // x1 = -1：胶囊 1 负端。
    {
      const v3 vec1 = v3_sub(cap1_pos, axis1);
      const float x2 = fminf(fmaxf((v + mb) / mc, -1.0f), 1.0f);
      const v3 vec2 = v3_add(cap2_pos, v3_scale(axis2, x2));
      const g5_primitive_contact contact = g5_sphere_sphere(vec1, cap1_radius, vec2, cap2_radius);
      if (contact.dist <= margin) {
        pair->contact[contact_count] = contact;
        contact_count += 1;
      }
    }

    // x2 = 1：胶囊 2 正端（仅在仍缺第 2 个接触时测试）。
    if (contact_count < 2) {
      const v3 vec2 = v3_add(cap2_pos, axis2);
      const float x1 = fminf(fmaxf((u - mb) / ma, -1.0f), 1.0f);
      const v3 vec1 = v3_add(cap1_pos, v3_scale(axis1, x1));
      const g5_primitive_contact contact = g5_sphere_sphere(vec1, cap1_radius, vec2, cap2_radius);
      if (contact.dist <= margin) {
        pair->contact[contact_count] = contact;
        contact_count += 1;
      }
    }

    // x2 = -1：胶囊 2 负端（仅在仍缺第 2 个接触时测试）。
    if (contact_count < 2) {
      const v3 vec2 = v3_sub(cap2_pos, axis2);
      const float x1 = fminf(fmaxf((u + mb) / ma, -1.0f), 1.0f);
      const v3 vec1 = v3_add(cap1_pos, v3_scale(axis1, x1));
      const g5_primitive_contact contact = g5_sphere_sphere(vec1, cap1_radius, vec2, cap2_radius);
      if (contact.dist <= margin) {
        pair->contact[contact_count] = contact;
      }
    }
  }
}

// 上游 `collision_primitive_core.sphere_cylinder`：侧面 / 端面 / 角点三分支。
// 端面分支复用 `plane_sphere` 后反转法线（pinned 注释 `flip normal after position calculation`）。
__device__ __forceinline__ g5_primitive_contact g5_sphere_cylinder(
    v3 sphere_pos, float sphere_radius, v3 cylinder_pos, v3 cylinder_axis, float cylinder_radius,
    float cylinder_half_height) {
  const v3 vec = v3_sub(sphere_pos, cylinder_pos);
  const float x = v3_dot(vec, cylinder_axis);

  const v3 a_proj = v3_scale(cylinder_axis, x);
  v3 p_proj = v3_sub(vec, a_proj);
  const float p_proj_sqr = v3_dot(p_proj, p_proj);

  bool collide_side = fabsf(x) < cylinder_half_height;
  bool collide_cap = p_proj_sqr < (cylinder_radius * cylinder_radius);

  if (collide_side && collide_cap) {
    const float dist_cap = cylinder_half_height - fabsf(x);
    const float dist_radius = cylinder_radius - sqrtf(p_proj_sqr);

    if (dist_cap < dist_radius) {
      collide_side = false;
    } else {
      collide_cap = false;
    }
  }

  // 侧面碰撞。
  if (collide_side) {
    const v3 pos_target = v3_add(cylinder_pos, a_proj);
    return g5_sphere_sphere(sphere_pos, sphere_radius, pos_target, cylinder_radius);
  }
  // 端面碰撞。
  if (collide_cap) {
    v3 pos_cap;
    v3 plane_normal;
    if (x > 0.0f) {
      // 顶面。
      pos_cap = v3_add(cylinder_pos, v3_scale(cylinder_axis, cylinder_half_height));
      plane_normal = cylinder_axis;
    } else {
      // 底面。
      pos_cap = v3_sub(cylinder_pos, v3_scale(cylinder_axis, cylinder_half_height));
      plane_normal = v3_scale(cylinder_axis, -1.0f);
    }

    float dist = 0.0f;
    v3 pos = v3_make(0.0f, 0.0f, 0.0f);
    g5_plane_sphere(plane_normal, pos_cap, sphere_pos, sphere_radius, &dist, &pos);
    g5_primitive_contact contact;
    contact.dist = dist;
    contact.pos = pos;
    contact.normal = v3_scale(plane_normal, -1.0f);
    return contact;
  }
  // 角点碰撞。
  const float inv_len = g5_primitive_safe_div(1.0f, sqrtf(p_proj_sqr));
  p_proj = v3_scale(p_proj, cylinder_radius * inv_len);

  // 上游 `wp.sign(x)`：x > 0 → 1，x < 0 → -1，x == 0 → 0。
  const float sign = (x > 0.0f) ? 1.0f : ((x < 0.0f) ? -1.0f : 0.0f);
  const v3 cap_offset = v3_scale(cylinder_axis, sign * cylinder_half_height);
  const v3 pos_corner = v3_add(v3_add(cylinder_pos, cap_offset), p_proj);

  return g5_sphere_sphere(sphere_pos, sphere_radius, pos_corner, 0.0f);
}
// G10@P5 B2 sphere/capsule 族 primitive 接触写入设备版。
// Adapted from frozen MJWarp `collision_primitive.py`
// （`sphere_sphere_wrapper` L357-429、`sphere_capsule_wrapper` L432-507、
// `capsule_capsule_wrapper` L510-597、`sphere_cylinder_wrapper` L915-1000），
// revision 71da24d956378a87a703b6e1442b13aec0c4ac29。Apache-2.0。
//
// 只做「取几何字段 → 调数值核心 → 逐接触 make_frame + write_contact」；margin/gap/condim/
// friction/... 已在 `d_contact_params` 里。写入门/顺序按 pinned，不加特判；0 新 ABI 字段。

// 上游 `sphere_sphere_wrapper`：单接触，`write_contact(id_=0)`。
__device__ __forceinline__ void g5_sphere_sphere_wrapper(const d_geom& sphere1,
                                                         const d_geom& sphere2,
                                                         const d_contact_params& params,
                                                         int naconmax, d_contact_out* out) {
  const g5_primitive_contact contact =
      g5_sphere_sphere(sphere1.pos, sphere1.size.x, sphere2.pos, sphere2.size.x);
  float frame[9];
  make_frame(contact.normal, frame);
  g5_write_contact(params, naconmax, 0, contact.dist, contact.pos, frame, out);
}

// 上游 `sphere_capsule_wrapper`：单接触，胶囊轴 = 旋转矩阵第三列。
__device__ __forceinline__ void g5_sphere_capsule_wrapper(const d_geom& sphere,
                                                          const d_geom& capsule,
                                                          const d_contact_params& params,
                                                          int naconmax, d_contact_out* out) {
  // 上游 `axis = vec3(cap.rot[0, 2], cap.rot[1, 2], cap.rot[2, 2])`（行优先 mat33 的第三列）。
  const v3 axis = v3_make(capsule.rot[2], capsule.rot[5], capsule.rot[8]);
  const g5_primitive_contact contact =
      g5_sphere_capsule(sphere.pos, sphere.size.x, capsule.pos, axis, capsule.size.x,
                        capsule.size.y);
  float frame[9];
  make_frame(contact.normal, frame);
  g5_write_contact(params, naconmax, 0, contact.dist, contact.pos, frame, out);
}

// 上游 `capsule_capsule_wrapper`：两接触（`id_ = 0/1`）；无效槽 `dist = wp.inf` 由
// `write_contact` 的 `detected` 门跳过，与上游计数语义一致。
__device__ __forceinline__ void g5_capsule_capsule_wrapper(const d_geom& cap1,
                                                           const d_geom& cap2,
                                                           const d_contact_params& params,
                                                           int naconmax, d_contact_out* out) {
  const v3 cap1_axis = v3_make(cap1.rot[2], cap1.rot[5], cap1.rot[8]);
  const v3 cap2_axis = v3_make(cap2.rot[2], cap2.rot[5], cap2.rot[8]);
  g5_primitive_pair pair;
  g5_capsule_capsule_core(cap1.pos, cap1_axis, cap1.size.x, cap1.size.y, cap2.pos, cap2_axis,
                          cap2.size.x, cap2.size.y, params.margin, &pair);
  for (int i = 0; i < 2; ++i) {
    const g5_primitive_contact& contact = pair.contact[i];
    float frame[9];
    make_frame(contact.normal, frame);
    g5_write_contact(params, naconmax, i, contact.dist, contact.pos, frame, out);
  }
}

// 上游 `sphere_cylinder_wrapper`：单接触，圆柱轴 = 旋转矩阵第三列。
__device__ __forceinline__ void g5_sphere_cylinder_wrapper(const d_geom& sphere,
                                                           const d_geom& cylinder,
                                                           const d_contact_params& params,
                                                           int naconmax, d_contact_out* out) {
  const v3 axis = v3_make(cylinder.rot[2], cylinder.rot[5], cylinder.rot[8]);
  const g5_primitive_contact contact =
      g5_sphere_cylinder(sphere.pos, sphere.size.x, cylinder.pos, axis, cylinder.size.x,
                         cylinder.size.y);
  float frame[9];
  make_frame(contact.normal, frame);
  g5_write_contact(params, naconmax, 0, contact.dist, contact.pos, frame, out);
}
// G10@P5 产品侧 CUDA 入口（NVRTC 运行时编译）。
//
// adapted from G5 独立探针 probe.cu（同 frozen 上游 revision 71da24d），Apache-2.0，
// 产品入口与探针共用同一批设备函数。
//
// 本文件只含产品入口需要的设备侧参数结构与六个 kernel：
//   - `mjwarp_collision_args`（原 `ProbeArgs`；M0a 起是冻结探针 `ProbeArgs` 的**超集**：
//     前 23 个指针与标量块顺序/类型与探针一致，仅在指针块末尾插入 `mesh_poly_args` sidecar
//     指针，总大小 496→504；探针 `ProbeArgs` 本身保持 496 冻结，不改探针/旧证据）
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

// B3/M0a：mesh polygon sidecar（方案 A）。9 个指针顺序与 pinned `collision_convex.py`
// L197-205 一致；9 个宿主数组全空时 `mjwarp_collision_args::mesh_poly_args == nullptr`
// （无 polygon 数据、mesh 多接触功能不启用、行为与改动前一致）。M0a 做 ABI 接线；
// M2 起由 `build_geom` 按 `geom_dataid` 解析出 `d_geom::mesh_polyadr` 与 9 个数组指针，
// 供 `multicontact_mesh.cu` 的 convex-vs-mesh 分支消费（sidecar 缺失 → 单接触回退）。
struct MjwarpMeshPolyArgs {
  const int* mesh_polynum;
  const int* mesh_polyadr;
  const float* mesh_polynormal;
  const int* mesh_polyvertadr;
  const int* mesh_polyvertnum;
  const int* mesh_polyvert;
  const int* mesh_polymapadr;
  const int* mesh_polymapnum;
  const int* mesh_polymap;
};

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
  // B3/M0a：mesh polygon sidecar 指针（9 数组全空时为 nullptr；M2 才消费）。
  const MjwarpMeshPolyArgs* mesh_poly_args;

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
  geom.dataid = -1;
  geom.mesh_polyadr = -1;
  geom.mesh.vert = nullptr;
  geom.mesh.graph = nullptr;
  g5_mesh_clear_poly(&geom.mesh);
  const int type = a.geom_type[geom_index];
  if (type == G5_MESH) {
    const int dataid = a.geom_dataid[geom_index];
    geom.vertadr = dataid >= 0 ? a.mesh_vertadr[dataid] : -1;
    geom.vertnum = dataid >= 0 ? a.mesh_vertnum[dataid] : -1;
    geom.graphadr = dataid >= 0 ? a.mesh_graphadr[dataid] : -1;
    geom.dataid = dataid;
    geom.mesh.vert = a.mesh_vert;
    geom.mesh.graph = a.mesh_graph;
    // M2：mesh polygon sidecar（9 指针，形状语义见 M0a 修正片的 validate_mesh_poly）。
    // 全空/缺失（nullptr）时保持 9 指针为空 → 多接触回退单接触（与 M0a 语义一致）。
    const MjwarpMeshPolyArgs* poly = a.mesh_poly_args;
    if (poly != nullptr && dataid >= 0) {
      geom.mesh.polynum = poly->mesh_polynum;
      geom.mesh.polyadr = poly->mesh_polyadr;
      geom.mesh.polynormal = poly->mesh_polynormal;
      geom.mesh.polyvertadr = poly->mesh_polyvertadr;
      geom.mesh.polyvertnum = poly->mesh_polyvertnum;
      geom.mesh.polyvert = poly->mesh_polyvert;
      geom.mesh.polymapadr = poly->mesh_polymapadr;
      geom.mesh.polymapnum = poly->mesh_polymapnum;
      geom.mesh.polymap = poly->mesh_polymap;
      geom.mesh_polyadr = poly->mesh_polyadr[dataid];
    }
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
// 已实现 {SPHERE,BOX}、{CAPSULE,BOX}、B1 plane 族 6 组合、B2 sphere/capsule 族 4 组合
// （兼容几何顺序互换）；其它组合不写接触。
// B3：按上游 `_add_geom_pair`（collision_driver.py L351-380）把 pair 规范化为类型序
// （type1 > type2 时交换 geom/type；相等类型保持调用方顺序），故 `contact.geom` 字段顺序
// 与上游一致；规范化对既有调用方（低类在前）是 no-op。0 新 ABI 字段/入口。
extern "C" __global__ void mjwarp_collision_primitive(mjwarp_collision_args a) {
  const int world = blockIdx.x;
  if (world >= a.nworld) return;
  const bool pair_swap = a.type1 > a.type2;
  const int type1 = pair_swap ? a.type2 : a.type1;
  const int type2 = pair_swap ? a.type1 : a.type2;
  const int geom_index1 = pair_swap ? a.g2 : a.g1;
  const int geom_index2 = pair_swap ? a.g1 : a.g2;

  d_contact_params params = build_params(a);
  params.worldid = world;
  // 上游 `contact_geom` 取规范化后的 pair 顺序（与 `_add_geom_pair` 写出的 geom pair 一致）。
  params.geoms[0] = geom_index1;
  params.geoms[1] = geom_index2;

  const d_geom geom1 = build_geom(a, world, geom_index1);
  const d_geom geom2 = build_geom(a, world, geom_index2);

  if (type1 == G5_SPHERE && type2 == G5_BOX) {
    g5_sphere_box_wrapper(geom1, geom2, params, a.naconmax, &a.out);
  } else if (type1 == G5_BOX && type2 == G5_SPHERE) {
    g5_sphere_box_wrapper(geom2, geom1, params, a.naconmax, &a.out);
  } else if (type1 == G5_CAPSULE && type2 == G5_BOX) {
    g5_capsule_box_wrapper(geom1, geom2, params, a.naconmax, &a.out);
  } else if (type1 == G5_BOX && type2 == G5_CAPSULE) {
    g5_capsule_box_wrapper(geom2, geom1, params, a.naconmax, &a.out);
  } else if (type1 == G5_SPHERE && type2 == G5_SPHERE) {
    // B2 sphere/capsule 族：pinned `_PRIMITIVE_COLLISIONS` 的 4 组合；wrapper 参数恒按
    // pinned key 顺序（sphere 在前、capsule 在前），几何互换时交换入参。
    g5_sphere_sphere_wrapper(geom1, geom2, params, a.naconmax, &a.out);
  } else if (type1 == G5_SPHERE && type2 == G5_CAPSULE) {
    g5_sphere_capsule_wrapper(geom1, geom2, params, a.naconmax, &a.out);
  } else if (type1 == G5_CAPSULE && type2 == G5_SPHERE) {
    g5_sphere_capsule_wrapper(geom2, geom1, params, a.naconmax, &a.out);
  } else if (type1 == G5_SPHERE && type2 == G5_CYLINDER) {
    g5_sphere_cylinder_wrapper(geom1, geom2, params, a.naconmax, &a.out);
  } else if (type1 == G5_CYLINDER && type2 == G5_SPHERE) {
    g5_sphere_cylinder_wrapper(geom2, geom1, params, a.naconmax, &a.out);
  } else if (type1 == G5_CAPSULE && type2 == G5_CAPSULE) {
    g5_capsule_capsule_wrapper(geom1, geom2, params, a.naconmax, &a.out);
  } else if (type1 == G5_PLANE || type2 == G5_PLANE) {
    // B1 plane 族：pinned `_PRIMITIVE_COLLISIONS` 的 6 组合（PLANE+SPHERE/CAPSULE/ELLIPSOID/
    // CYLINDER/BOX/MESH）；wrapper 第一个参数恒为 plane。mesh 只用 mesh 顶点/图（a.mesh_vert /
    // a.mesh_graph），不需要新 ABI 字段。写入门/顺序由各 wrapper 按 pinned 逐行实现，不在此加特判。
    const bool plane_first = type1 == G5_PLANE;
    const d_geom& plane = plane_first ? geom1 : geom2;
    const d_geom& other = plane_first ? geom2 : geom1;
    const int other_type = plane_first ? type2 : type1;
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
