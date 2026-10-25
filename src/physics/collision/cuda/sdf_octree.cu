// G10@P5 G5：SDF octree 采样设备版，与 host `sdf/octree.rs` 逐式对应。
// Adapted from frozen MJWarp collision_sdf.py, revision 71da24d. Apache-2.0.
#pragma once
#include "common.cu"

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
